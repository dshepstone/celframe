use std::io::{Cursor, Write};

use celframe_core::{
    Drawing, Exposure, Layer, Project, SCHEMA_VERSION, Scene, Stroke, StrokeId, StrokeSample,
};
use celframe_format::{FormatError, load_project, read_project, save_project, write_project};
use zip::write::SimpleFileOptions;

fn sample_project() -> Project {
    let mut project = Project::new("Bouncing ball");
    project.scenes.push(Scene::new("Shot 2"));
    let layer = &mut project.scenes[0].layers[0];
    let mut drawing = Drawing::new();
    drawing.strokes.push(Stroke {
        id: StrokeId::new(),
        base_width: 4.25,
        color: [10, 20, 30, 255],
        samples: vec![
            StrokeSample {
                x: 0.1 + 0.2,
                y: -1e-300,
                pressure: 0.1,
            },
            StrokeSample {
                x: 1.0 / 3.0,
                y: 1234567.891,
                pressure: 1.0,
            },
        ],
    });
    let id = drawing.id;
    layer.drawings.push(drawing);
    layer.insert_exposure(Exposure::new(3, 2, id)).unwrap();
    project.scenes[0].layers.push(Layer::new("Top"));
    project
}

fn to_bytes(project: &Project) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    write_project(project, &mut buffer).unwrap();
    buffer.into_inner()
}

fn archive_with(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, contents) in entries {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(contents.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

/// A valid file with its scene JSON edited by `edit`.
fn with_edited_scene(edit: impl FnOnce(&mut serde_json::Value)) -> Vec<u8> {
    let project = sample_project();
    let manifest = serde_json::json!({
        "schemaVersion": SCHEMA_VERSION,
        "id": project.id,
        "name": project.name,
        "scenes": [project.scenes[0].id],
    });
    let mut scene = serde_json::to_value(&project.scenes[0]).unwrap();
    edit(&mut scene);
    let scene_entry = format!("scenes/{}.json", project.scenes[0].id.as_uuid());
    archive_with(&[
        ("project.json", &manifest.to_string()),
        (&scene_entry, &scene.to_string()),
    ])
}

#[test]
fn a_project_round_trips_exactly() {
    let project = sample_project();
    let loaded = read_project(Cursor::new(to_bytes(&project))).unwrap();
    assert_eq!(loaded, project);
}

#[test]
fn saving_the_same_project_twice_gives_identical_bytes() {
    let project = sample_project();
    assert_eq!(to_bytes(&project), to_bytes(&project));
}

#[test]
fn the_archive_has_the_documented_layout() {
    let project = sample_project();
    let mut archive = zip::ZipArchive::new(Cursor::new(to_bytes(&project))).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
        .collect();
    assert_eq!(names[0], "project.json");
    assert_eq!(
        names[1],
        format!("scenes/{}.json", project.scenes[0].id.as_uuid())
    );
    assert_eq!(
        names[2],
        format!("scenes/{}.json", project.scenes[1].id.as_uuid())
    );
    assert_eq!(names[3], "assets/");

    let manifest: serde_json::Value =
        serde_json::from_reader(archive.by_name("project.json").unwrap()).unwrap();
    assert_eq!(manifest["schemaVersion"], SCHEMA_VERSION);
    assert_eq!(manifest["scenes"].as_array().unwrap().len(), 2);
}

#[test]
fn saved_json_uses_camel_case_field_names() {
    let json = serde_json::to_value(&sample_project().scenes[0]).unwrap();
    assert!(json.get("frameRate").is_some());
    assert!(
        json["layers"][0]["exposures"][0]
            .get("startFrame")
            .is_some()
    );
    assert!(
        json["layers"][0]["drawings"][0]["strokes"][0]
            .get("baseWidth")
            .is_some()
    );
}

#[test]
fn save_and_load_through_a_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ball.celf");
    let project = sample_project();
    save_project(&project, &path).unwrap();
    assert_eq!(load_project(&path).unwrap(), project);
}

#[test]
fn saving_over_a_file_replaces_it_and_leaves_no_temp_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ball.celf");
    save_project(&Project::new("Old"), &path).unwrap();
    let project = sample_project();
    save_project(&project, &path).unwrap();
    assert_eq!(load_project(&path).unwrap(), project);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn a_failed_save_keeps_the_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ball.celf");
    let original = sample_project();
    save_project(&original, &path).unwrap();

    let mut broken = sample_project();
    broken.scenes[0].layers[0].drawings[0].strokes[0].samples[0].x = f64::NAN;
    assert!(matches!(
        save_project(&broken, &path),
        Err(FormatError::InvalidDocument(_))
    ));
    assert_eq!(load_project(&path).unwrap(), original);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn not_a_zip_is_an_error() {
    let result = read_project(Cursor::new(b"not a zip".to_vec()));
    assert!(matches!(result, Err(FormatError::Zip(_))));
}

#[test]
fn a_missing_manifest_is_an_error() {
    let result = read_project(Cursor::new(archive_with(&[("other.json", "{}")])));
    assert!(matches!(result, Err(FormatError::MissingEntry(name)) if name == "project.json"));
}

#[test]
fn a_missing_scene_is_an_error() {
    let project = sample_project();
    let manifest = serde_json::json!({
        "schemaVersion": SCHEMA_VERSION,
        "id": project.id,
        "name": project.name,
        "scenes": [project.scenes[0].id],
    });
    let result = read_project(Cursor::new(archive_with(&[(
        "project.json",
        &manifest.to_string(),
    )])));
    assert!(matches!(result, Err(FormatError::MissingEntry(_))));
}

#[test]
fn damaged_json_is_an_error() {
    let result = read_project(Cursor::new(archive_with(&[("project.json", "{oops")])));
    assert!(matches!(result, Err(FormatError::Json(_))));
}

#[test]
fn a_project_with_no_scenes_is_rejected() {
    let project = Project::new("A");
    let manifest = serde_json::json!({
        "schemaVersion": SCHEMA_VERSION,
        "id": project.id,
        "name": project.name,
        "scenes": [],
    });
    let result = read_project(Cursor::new(archive_with(&[(
        "project.json",
        &manifest.to_string(),
    )])));
    assert!(matches!(result, Err(FormatError::InvalidDocument(_))));
}

#[test]
fn a_scene_whose_id_differs_from_its_entry_is_rejected() {
    let bytes = with_edited_scene(|scene| {
        scene["id"] = serde_json::json!("01900000-0000-7000-8000-000000000001");
    });
    assert!(matches!(
        read_project(Cursor::new(bytes)),
        Err(FormatError::InvalidDocument(_))
    ));
}

#[test]
fn overlapping_exposures_in_a_file_are_rejected() {
    let bytes = with_edited_scene(|scene| {
        let exposures = scene["layers"][0]["exposures"].as_array_mut().unwrap();
        let copy = exposures[0].clone();
        exposures.push(copy);
    });
    assert!(matches!(
        read_project(Cursor::new(bytes)),
        Err(FormatError::InvalidDocument(_))
    ));
}

#[test]
fn out_of_range_pressure_in_a_file_is_rejected() {
    let bytes = with_edited_scene(|scene| {
        scene["layers"][0]["drawings"][0]["strokes"][0]["samples"][0]["pressure"] =
            serde_json::json!(1.5);
    });
    assert!(matches!(
        read_project(Cursor::new(bytes)),
        Err(FormatError::InvalidDocument(_))
    ));
}
