//! `.celf` file reading and writing. The format is specified in `docs/file-format.md`.

mod migrate;
#[cfg(test)]
mod migration_tests;
mod validate;

pub use migrate::{Migration, RawDocument};

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;

use celframe_core::{
    Project, ProjectId, SCHEMA_VERSION, Scene, SceneId, SchemaCompatibility, schema_compatibility,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const MANIFEST_ENTRY: &str = "project.json";
const ASSETS_ENTRY: &str = "assets/";

/// Most JSON we'll decompress from one file, so a crafted file can't exhaust memory.
const MAX_JSON_BYTES: u64 = 1 << 30;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema_version: u32,
    id: ProjectId,
    name: String,
    scenes: Vec<SceneId>,
}

fn scene_entry(id: SceneId) -> String {
    format!("scenes/{}.json", id.as_uuid())
}

/// Writes a project as a `.celf` archive. The same project always gives the same bytes.
pub fn write_project<W: Write + Seek>(project: &Project, writer: W) -> Result<(), FormatError> {
    validate::validate(project)?;

    // The default timestamp is fixed, which keeps output reproducible.
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut zip = ZipWriter::new(writer);

    let manifest = Manifest {
        schema_version: SCHEMA_VERSION,
        id: project.id,
        name: project.name.clone(),
        scenes: project.scenes.iter().map(|s| s.id).collect(),
    };
    zip.start_file(MANIFEST_ENTRY, options)?;
    zip.write_all(&serde_json::to_vec_pretty(&manifest)?)?;

    for scene in &project.scenes {
        zip.start_file(scene_entry(scene.id), options)?;
        zip.write_all(&serde_json::to_vec_pretty(scene)?)?;
    }

    zip.add_directory(ASSETS_ENTRY, options)?;
    zip.finish()?;
    Ok(())
}

/// Reads a `.celf` archive, upgrading older files and rejecting damaged or unsupported ones.
pub fn read_project<R: Read + Seek>(reader: R) -> Result<Project, FormatError> {
    read_with(reader, SCHEMA_VERSION, migrate::MIGRATIONS)
}

fn read_with<R: Read + Seek>(
    reader: R,
    current_version: u32,
    migrations: &[Migration],
) -> Result<Project, FormatError> {
    let mut archive = ZipArchive::new(reader)?;
    let mut remaining = MAX_JSON_BYTES;

    let manifest: Value =
        serde_json::from_slice(&read_entry(&mut archive, MANIFEST_ENTRY, &mut remaining)?)?;
    let version = manifest
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0);
    match schema_compatibility(version, current_version) {
        SchemaCompatibility::Current | SchemaCompatibility::NeedsMigration => {}
        SchemaCompatibility::TooNew => return Err(FormatError::TooNew(version)),
        SchemaCompatibility::Invalid => {
            return Err(FormatError::InvalidDocument(
                "the schema version is missing or invalid".to_owned(),
            ));
        }
    }

    let mut scenes = BTreeMap::new();
    for name in scene_entry_names(&mut archive)? {
        let bytes = read_entry(&mut archive, &name, &mut remaining)?;
        scenes.insert(name, serde_json::from_slice(&bytes)?);
    }
    let mut document = RawDocument { manifest, scenes };
    migrate::migrate(&mut document, version, current_version, migrations)?;

    let manifest: Manifest = serde_json::from_value(document.manifest)?;
    let mut project_scenes = Vec::with_capacity(manifest.scenes.len());
    for id in manifest.scenes {
        let name = scene_entry(id);
        let json = document
            .scenes
            .remove(&name)
            .ok_or(FormatError::MissingEntry(name))?;
        let scene: Scene = serde_json::from_value(json)?;
        if scene.id != id {
            return Err(FormatError::InvalidDocument(
                "a scene's ID doesn't match its entry".to_owned(),
            ));
        }
        project_scenes.push(scene);
    }

    let project = Project {
        id: manifest.id,
        name: manifest.name,
        scenes: project_scenes,
    };
    validate::validate(&project)?;
    Ok(project)
}

fn scene_entry_names<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<Vec<String>, FormatError> {
    let mut names = Vec::new();
    for index in 0..archive.len() {
        let name = archive.by_index_raw(index)?.name()?.to_string();
        if name.starts_with("scenes/") && name.ends_with(".json") {
            names.push(name);
        }
    }
    Ok(names)
}

fn read_entry<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
    remaining: &mut u64,
) -> Result<Vec<u8>, FormatError> {
    let entry = archive.by_name(name).map_err(|error| match error {
        zip::result::ZipError::FileNotFound => FormatError::MissingEntry(name.to_owned()),
        other => FormatError::Zip(other),
    })?;
    let mut bytes = Vec::new();
    entry.take(*remaining + 1).read_to_end(&mut bytes)?;
    *remaining = remaining
        .checked_sub(bytes.len() as u64)
        .ok_or_else(|| FormatError::InvalidDocument("this file's data is too large".to_owned()))?;
    Ok(bytes)
}

/// Saves atomically: an existing file stays intact unless the new one is fully written.
pub fn save_project(project: &Project, path: &Path) -> Result<(), FormatError> {
    // Same folder as the target, because a rename can't cross drives.
    let file_name = path
        .file_name()
        .ok_or_else(|| FormatError::InvalidDocument("the save path has no file name".to_owned()))?
        .to_string_lossy();
    let temp_path = path.with_file_name(format!(".{file_name}.{}.tmp", uuid::Uuid::now_v7()));

    let result = write_and_replace(project, &temp_path, path);
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn write_and_replace(project: &Project, temp_path: &Path, path: &Path) -> Result<(), FormatError> {
    let mut writer = BufWriter::new(File::create_new(temp_path)?);
    write_project(project, &mut writer)?;
    let file = writer.into_inner().map_err(|e| e.into_error())?;
    file.sync_all()?;
    drop(file);
    fs::rename(temp_path, path)?;
    Ok(())
}

pub fn load_project(path: &Path) -> Result<Project, FormatError> {
    read_project(BufReader::new(File::open(path)?))
}

#[derive(Debug)]
pub enum FormatError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Json(serde_json::Error),
    MissingEntry(String),
    /// Saved by a newer build of Celframe.
    TooNew(u32),
    InvalidDocument(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "couldn't read or write the file: {error}"),
            Self::Zip(error) => write!(f, "this isn't a valid .celf file: {error}"),
            Self::Json(error) => write!(f, "this .celf file is damaged: {error}"),
            Self::MissingEntry(name) => write!(f, "this .celf file is missing {name}"),
            Self::TooNew(version) => write!(
                f,
                "this file uses schema version {version}, which needs a newer version of Celframe"
            ),
            Self::InvalidDocument(reason) => write!(f, "this document isn't valid: {reason}"),
        }
    }
}

impl std::error::Error for FormatError {}

impl From<std::io::Error> for FormatError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<zip::result::ZipError> for FormatError {
    fn from(error: zip::result::ZipError) -> Self {
        Self::Zip(error)
    }
}

impl From<serde_json::Error> for FormatError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
