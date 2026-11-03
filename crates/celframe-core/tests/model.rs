use celframe_core::{Drawing, DrawingId, Exposure, ExposureError, Layer, Project, Scene, StrokeId};

fn layer_with_two_exposures() -> (Layer, DrawingId, DrawingId) {
    let mut layer = Layer::new("Ink");
    let a = Drawing::new();
    let b = Drawing::new();
    let (a_id, b_id) = (a.id, b.id);
    layer.drawings.extend([a, b]);
    layer.insert_exposure(Exposure::new(0, 2, a_id)).unwrap();
    layer.insert_exposure(Exposure::new(4, 1, b_id)).unwrap();
    (layer, a_id, b_id)
}

#[test]
fn new_ids_are_unique_version_7_and_time_ordered() {
    let first = StrokeId::new();
    let second = StrokeId::new();
    assert_ne!(first, second);
    assert_eq!(first.as_uuid().get_version_num(), 7);
    assert!(first < second);
}

#[test]
fn ids_serialize_as_plain_strings_and_round_trip() {
    let id = StrokeId::from_u128(0x0190_0000_0000_7000_8000_0000_0000_0001);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "\"01900000-0000-7000-8000-000000000001\"");
    assert_eq!(serde_json::from_str::<StrokeId>(&json).unwrap(), id);
}

#[test]
fn new_project_starts_with_one_scene_and_one_layer() {
    let project = Project::new("Bouncing ball");
    assert_eq!(project.name, "Bouncing ball");
    assert_eq!(project.scenes.len(), 1);
    let scene = &project.scenes[0];
    assert_eq!(
        (scene.width, scene.height, scene.frame_rate),
        (1920, 1080, 24)
    );
    assert_eq!(scene.layers.len(), 1);
}

#[test]
fn drawing_at_follows_exposures_and_holds() {
    let (layer, a, b) = layer_with_two_exposures();
    assert_eq!(layer.drawing_at(0).map(|d| d.id), Some(a));
    assert_eq!(layer.drawing_at(1).map(|d| d.id), Some(a));
    assert_eq!(layer.drawing_at(2).map(|d| d.id), None);
    assert_eq!(layer.drawing_at(4).map(|d| d.id), Some(b));
    assert_eq!(layer.drawing_at(5).map(|d| d.id), None);
}

#[test]
fn exposures_stay_sorted_by_start_frame() {
    let (mut layer, a, _) = layer_with_two_exposures();
    layer.insert_exposure(Exposure::new(2, 1, a)).unwrap();
    let starts: Vec<u32> = layer.exposures.iter().map(|e| e.start_frame).collect();
    assert_eq!(starts, [0, 2, 4]);
}

#[test]
fn overlapping_exposures_are_rejected_and_change_nothing() {
    let (mut layer, a, _) = layer_with_two_exposures();
    let result = layer.insert_exposure(Exposure::new(1, 1, a));
    assert_eq!(result, Err(ExposureError::Overlap));
    assert_eq!(layer.exposures.len(), 2);
}

#[test]
fn zero_length_exposures_are_rejected() {
    let (mut layer, a, _) = layer_with_two_exposures();
    assert_eq!(
        layer.insert_exposure(Exposure::new(10, 0, a)),
        Err(ExposureError::ZeroLength)
    );
}

#[test]
fn exposures_must_reference_a_drawing_on_the_layer() {
    let (mut layer, _, _) = layer_with_two_exposures();
    assert_eq!(
        layer.insert_exposure(Exposure::new(10, 1, Drawing::new().id)),
        Err(ExposureError::UnknownDrawing)
    );
}

#[test]
fn an_exposure_reaching_the_last_frame_number_does_not_overflow() {
    let (mut layer, a, _) = layer_with_two_exposures();
    layer
        .insert_exposure(Exposure::new(u32::MAX - 1, 5, a))
        .unwrap();
    assert_eq!(layer.drawing_at(u32::MAX).map(|d| d.id), Some(a));
}

#[test]
fn scene_serializes_and_round_trips() {
    let scene = Scene::new("Shot 1");
    let json = serde_json::to_string(&scene).unwrap();
    assert_eq!(serde_json::from_str::<Scene>(&json).unwrap(), scene);
}
