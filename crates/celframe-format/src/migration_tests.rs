use std::io::{Cursor, Write};

use celframe_core::{Project, SCHEMA_VERSION};
use serde_json::{Value, json};
use zip::write::SimpleFileOptions;

use crate::migrate::{MIGRATIONS, Migration, RawDocument};
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

/// The real migrations followed by `extra`, so `extra[0]` upgrades from `CURRENT`.
fn steps(extra: &[Migration]) -> Vec<Migration> {
    MIGRATIONS.iter().chain(extra).copied().collect()
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
    let loaded = read_with(Cursor::new(bytes), CURRENT + 1, &steps(migrations)).unwrap();
    assert_eq!(loaded, project);
}

#[test]
fn migrations_run_in_order_across_several_versions() {
    fn does_nothing(_: &mut RawDocument) -> Result<(), String> {
        Ok(())
    }
    let (project, bytes) = old_file();
    let migrations: &[Migration] = &[rename_old_fields, does_nothing];
    let loaded = read_with(Cursor::new(bytes), CURRENT + 2, &steps(migrations)).unwrap();
    assert_eq!(loaded, project);
}

#[test]
fn a_missing_migration_step_is_an_error() {
    let (_, bytes) = old_file();
    let result = read_with(Cursor::new(bytes), CURRENT + 1, MIGRATIONS);
    assert!(matches!(result, Err(FormatError::InvalidDocument(_))));
}

#[test]
fn a_failing_migration_is_an_error() {
    fn fails(_: &mut RawDocument) -> Result<(), String> {
        Err("nope".to_owned())
    }
    let (_, bytes) = old_file();
    let migrations: &[Migration] = &[fails];
    let result = read_with(Cursor::new(bytes), CURRENT + 1, &steps(migrations));
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

/// A version 1 file as Celframe 1 wrote it: strokes have no `art`.
fn version_1_file() -> (Project, Vec<u8>) {
    let mut project = Project::starter("Old ink");
    let drawing = &mut project.scenes[0].layers[0].drawings[0];
    drawing.strokes.push(celframe_core::Stroke {
        id: celframe_core::StrokeId::new(),
        base_width: 3.0,
        color: [1, 2, 3, 255],
        art: celframe_core::Art::Line,
        samples: vec![celframe_core::StrokeSample {
            x: 1.5,
            y: -2.5,
            pressure: 0.5,
        }],
    });
    let scene = &project.scenes[0];
    let manifest = json!({
        "schemaVersion": 1,
        "id": project.id,
        "name": project.name,
        "scenes": [scene.id],
    });
    let mut scene_json = serde_json::to_value(scene).unwrap();
    let stroke = &mut scene_json["layers"][0]["drawings"][0]["strokes"][0];
    stroke.as_object_mut().unwrap().remove("art").unwrap();
    // Nor do layers have `opacity`, nor scenes a `camera`.
    scene_json["layers"][0]
        .as_object_mut()
        .unwrap()
        .remove("opacity")
        .unwrap();
    scene_json
        .as_object_mut()
        .unwrap()
        .remove("camera")
        .unwrap();

    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    zip.start_file("project.json", options).unwrap();
    zip.write_all(manifest.to_string().as_bytes()).unwrap();
    zip.start_file(format!("scenes/{}.json", scene.id.as_uuid()), options)
        .unwrap();
    zip.write_all(scene_json.to_string().as_bytes()).unwrap();
    (project, zip.finish().unwrap().into_inner())
}

#[test]
fn a_version_1_file_opens_with_every_stroke_as_line_art() {
    let (project, bytes) = version_1_file();
    assert_eq!(read_project(Cursor::new(bytes)).unwrap(), project);
}

#[test]
fn a_version_3_scene_opens_with_the_standard_camera() {
    let project = Project::starter("Before cameras");
    let scene = &project.scenes[0];
    let manifest = json!({
        "schemaVersion": 3,
        "id": project.id,
        "name": project.name,
        "scenes": [scene.id],
    });
    let mut scene_json = serde_json::to_value(scene).unwrap();
    scene_json
        .as_object_mut()
        .unwrap()
        .remove("camera")
        .unwrap();
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    zip.start_file("project.json", options).unwrap();
    zip.write_all(manifest.to_string().as_bytes()).unwrap();
    zip.start_file(format!("scenes/{}.json", scene.id.as_uuid()), options)
        .unwrap();
    zip.write_all(scene_json.to_string().as_bytes()).unwrap();
    let bytes = zip.finish().unwrap().into_inner();
    let loaded = read_project(Cursor::new(bytes)).unwrap();
    assert_eq!(loaded, project);
    assert_eq!(loaded.scenes[0].camera, celframe_core::Camera::default());
}
