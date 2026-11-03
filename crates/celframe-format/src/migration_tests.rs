use std::io::{Cursor, Write};

use celframe_core::{Project, SCHEMA_VERSION};
use serde_json::{Value, json};
use zip::write::SimpleFileOptions;

use crate::migrate::{Migration, RawDocument};
use crate::{FormatError, read_project, read_with};

const CURRENT: u32 = SCHEMA_VERSION;

/// Pretends the current schema once used `title`, and scenes used `fps`.
fn old_file() -> (Project, Vec<u8>) {
    let project = Project::new("Old");
    let scene = &project.scenes[0];
    let manifest = json!({
        "schemaVersion": CURRENT,
        "id": project.id,
        "title": project.name,
        "scenes": [scene.id],
    });
    let mut scene_json = serde_json::to_value(scene).unwrap();
    let rate = scene_json
        .as_object_mut()
        .unwrap()
        .remove("frameRate")
        .unwrap();
    scene_json["fps"] = rate;

    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    zip.start_file("project.json", options).unwrap();
    zip.write_all(manifest.to_string().as_bytes()).unwrap();
    zip.start_file(format!("scenes/{}.json", scene.id.as_uuid()), options)
        .unwrap();
    zip.write_all(scene_json.to_string().as_bytes()).unwrap();
    (project, zip.finish().unwrap().into_inner())
}

fn rename_old_fields(document: &mut RawDocument) -> Result<(), String> {
    let manifest = document.manifest.as_object_mut().ok_or("bad manifest")?;
    let title = manifest.remove("title").ok_or("no title")?;
    manifest.insert("name".to_owned(), title);
    for scene in document.scenes.values_mut() {
        let scene = scene.as_object_mut().ok_or("bad scene")?;
        let fps = scene.remove("fps").ok_or("no fps")?;
        scene.insert("frameRate".to_owned(), fps);
    }
    Ok(())
}

fn with_version(version: Value) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("project.json", SimpleFileOptions::default())
        .unwrap();
    let manifest = json!({ "schemaVersion": version });
    zip.write_all(manifest.to_string().as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

#[test]
fn an_older_file_is_migrated_then_loaded() {
    let (project, bytes) = old_file();
    let migrations: &[Migration] = &[rename_old_fields];
    let loaded = read_with(Cursor::new(bytes), CURRENT + 1, migrations).unwrap();
    assert_eq!(loaded, project);
}

#[test]
fn migrations_run_in_order_across_several_versions() {
    fn does_nothing(_: &mut RawDocument) -> Result<(), String> {
        Ok(())
    }
    let (project, bytes) = old_file();
    let migrations: &[Migration] = &[rename_old_fields, does_nothing];
    let loaded = read_with(Cursor::new(bytes), CURRENT + 2, migrations).unwrap();
    assert_eq!(loaded, project);
}

#[test]
fn a_missing_migration_step_is_an_error() {
    let (_, bytes) = old_file();
    let result = read_with(Cursor::new(bytes), CURRENT + 1, &[]);
    assert!(matches!(result, Err(FormatError::InvalidDocument(_))));
}

#[test]
fn a_failing_migration_is_an_error() {
    fn fails(_: &mut RawDocument) -> Result<(), String> {
        Err("nope".to_owned())
    }
    let (_, bytes) = old_file();
    let migrations: &[Migration] = &[fails];
    let result = read_with(Cursor::new(bytes), CURRENT + 1, migrations);
    assert!(matches!(result, Err(FormatError::InvalidDocument(reason)) if reason.contains("nope")));
}

#[test]
fn a_newer_file_is_refused_with_its_version() {
    let result = read_project(Cursor::new(with_version(json!(CURRENT + 1))));
    assert!(matches!(result, Err(FormatError::TooNew(v)) if v == CURRENT + 1));
}

#[test]
fn version_zero_or_a_missing_version_is_invalid() {
    for version in [json!(0), json!("one"), Value::Null] {
        let result = read_project(Cursor::new(with_version(version)));
        assert!(matches!(result, Err(FormatError::InvalidDocument(_))));
    }
}
