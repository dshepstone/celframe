use celframe_core::{
    Command, CommandError, Drawing, DrawingId, Exposure, ExposureError, Layer, LayerId, Project,
    Scene, SceneId, Stroke, StrokeId, StrokeSample,
};

struct Fixture {
    project: Project,
    scene: SceneId,
    layer: LayerId,
    drawing: DrawingId,
    stroke: StrokeId,
}

fn stroke() -> Stroke {
    Stroke {
        id: StrokeId::new(),
        base_width: 4.0,
        color: [0, 0, 0, 255],
        art: celframe_core::Art::Line,
        samples: vec![StrokeSample {
            x: 1.0,
            y: 2.0,
            pressure: 0.5,
        }],
    }
}

/// A project with two layers, one exposed drawing, one spare drawing and one stroke.
fn fixture() -> Fixture {
    let mut project = Project::new("A");
    let scene = &mut project.scenes[0];
    let scene_id = scene.id;
    scene.layers.push(Layer::new("Top"));
    let layer = &mut scene.layers[0];
    let layer_id = layer.id;
    let mut drawing = Drawing::new();
    let existing = stroke();
    let stroke_id = existing.id;
    drawing.strokes.push(existing);
    let drawing_id = drawing.id;
    layer.drawings.push(drawing);
    layer.drawings.push(Drawing::new());
    layer
        .insert_exposure(Exposure::new(0, 3, drawing_id))
        .unwrap();
    Fixture {
        project,
        scene: scene_id,
        layer: layer_id,
        drawing: drawing_id,
        stroke: stroke_id,
    }
}

/// Apply, undo and redo must each land exactly on the expected state.
fn assert_reversible(mut project: Project, command: Command) {
    let before = project.clone();
    let inverse = command.apply(&mut project).unwrap();
    let after = project.clone();
    assert_ne!(before, after, "command should change the project");
    let redo = inverse.apply(&mut project).unwrap();
    assert_eq!(project, before, "undo should restore the project");
    redo.apply(&mut project).unwrap();
    assert_eq!(project, after, "redo should restore the edit");
}

macro_rules! reversible {
    ($name:ident, $f:ident => $command:expr) => {
        #[test]
        fn $name() {
            let $f = fixture();
            let command = $command;
            assert_reversible($f.project, command);
        }
    };
}

reversible!(rename_project, _f => Command::RenameProject { name: "B".into() });
reversible!(add_scene, _f => Command::AddScene { scene: Scene::new("New"), index: 0 });
reversible!(rename_scene, f => Command::RenameScene { scene: f.scene, name: "B".into() });
reversible!(add_layer, f => Command::AddLayer { scene: f.scene, layer: Layer::new("New"), index: 1 });
reversible!(remove_layer, f => Command::RemoveLayer { scene: f.scene, layer: f.layer });
reversible!(move_layer, f => Command::MoveLayer { scene: f.scene, layer: f.layer, index: 1 });
reversible!(rename_layer, f => Command::RenameLayer { scene: f.scene, layer: f.layer, name: "B".into() });
reversible!(hide_layer, f => Command::SetLayerVisible { scene: f.scene, layer: f.layer, visible: false });
reversible!(lock_layer, f => Command::SetLayerLocked { scene: f.scene, layer: f.layer, locked: true });
reversible!(add_drawing, f => Command::AddDrawing { scene: f.scene, layer: f.layer, drawing: Drawing::new(), index: 1 });
reversible!(insert_exposure, f => Command::InsertExposure { scene: f.scene, layer: f.layer, exposure: Exposure::new(5, 2, f.drawing) });
reversible!(remove_exposure, f => Command::RemoveExposure { scene: f.scene, layer: f.layer, start_frame: 0 });
reversible!(add_stroke, f => Command::AddStroke { scene: f.scene, layer: f.layer, drawing: f.drawing, stroke: stroke(), index: 0 });
reversible!(remove_stroke, f => Command::RemoveStroke { scene: f.scene, layer: f.layer, drawing: f.drawing, stroke: f.stroke });

#[test]
fn remove_scene_restores_its_position() {
    let mut project = Project::new("A");
    let second = Scene::new("Second");
    let id = second.id;
    project.scenes.push(second);
    project.scenes.push(Scene::new("Third"));
    assert_reversible(project, Command::RemoveScene { scene: id });
}

#[test]
fn remove_unused_drawing_restores_its_position() {
    let f = fixture();
    let spare = f.project.scenes[0].layers[0].drawings[1].id;
    assert_reversible(
        f.project,
        Command::RemoveDrawing {
            scene: f.scene,
            layer: f.layer,
            drawing: spare,
        },
    );
}

/// Failed commands must report the expected error and change nothing.
fn assert_rejected(mut project: Project, command: Command, expected: CommandError) {
    let before = project.clone();
    assert_eq!(command.apply(&mut project), Err(expected));
    assert_eq!(project, before);
}

#[test]
fn the_last_scene_cannot_be_removed() {
    let f = fixture();
    assert_rejected(
        f.project,
        Command::RemoveScene { scene: f.scene },
        CommandError::LastScene,
    );
}

#[test]
fn a_drawing_in_use_cannot_be_removed() {
    let f = fixture();
    assert_rejected(
        f.project,
        Command::RemoveDrawing {
            scene: f.scene,
            layer: f.layer,
            drawing: f.drawing,
        },
        CommandError::DrawingInUse,
    );
}

#[test]
fn unknown_items_are_rejected() {
    let f = fixture();
    assert_rejected(
        f.project,
        Command::RemoveLayer {
            scene: f.scene,
            layer: LayerId::new(),
        },
        CommandError::NotFound,
    );
}

#[test]
fn out_of_range_positions_are_rejected() {
    let f = fixture();
    assert_rejected(
        f.project,
        Command::AddLayer {
            scene: f.scene,
            layer: Layer::new("New"),
            index: 99,
        },
        CommandError::InvalidIndex,
    );
    let f = fixture();
    assert_rejected(
        f.project,
        Command::MoveLayer {
            scene: f.scene,
            layer: f.layer,
            index: 2,
        },
        CommandError::InvalidIndex,
    );
}

#[test]
fn duplicate_ids_are_rejected() {
    let f = fixture();
    let existing = f.project.scenes[0].layers[0].clone();
    assert_rejected(
        f.project,
        Command::AddLayer {
            scene: f.scene,
            layer: existing,
            index: 0,
        },
        CommandError::DuplicateId,
    );
}

#[test]
fn invalid_exposures_are_rejected() {
    let f = fixture();
    assert_rejected(
        f.project,
        Command::InsertExposure {
            scene: f.scene,
            layer: f.layer,
            exposure: Exposure::new(1, 1, f.drawing),
        },
        CommandError::Exposure(ExposureError::Overlap),
    );
}

reversible!(batch, f => Command::Batch { commands: vec![
    Command::RemoveStroke { scene: f.scene, layer: f.layer, drawing: f.drawing, stroke: f.stroke },
    Command::AddStroke { scene: f.scene, layer: f.layer, drawing: f.drawing, stroke: stroke(), index: 0 },
    Command::RenameLayer { scene: f.scene, layer: f.layer, name: "B".into() },
] });

#[test]
fn a_failing_batch_leaves_the_project_unchanged() {
    let f = fixture();
    let command = Command::Batch {
        commands: vec![
            Command::RenameLayer {
                scene: f.scene,
                layer: f.layer,
                name: "B".into(),
            },
            Command::RemoveScene {
                scene: SceneId::new(),
            },
        ],
    };
    let mut project = f.project.clone();
    assert_eq!(command.apply(&mut project), Err(CommandError::NotFound));
    assert_eq!(project, f.project);
}

fn opacity_key(frame: u32, value: f64) -> celframe_core::Key {
    celframe_core::Key {
        frame,
        value,
        interpolation: celframe_core::Interpolation::Ease,
    }
}

reversible!(set_key, f => Command::SetKey { scene: f.scene, layer: f.layer, property: celframe_core::LayerProperty::Opacity, key: opacity_key(3, 0.5) });
reversible!(set_base_value, f => Command::SetBaseValue { scene: f.scene, layer: f.layer, property: celframe_core::LayerProperty::Opacity, value: 0.25 });

#[test]
fn replacing_and_removing_keys_reverse_exactly() {
    let f = fixture();
    let mut project = f.project.clone();
    let add = |frame, value| Command::SetKey {
        scene: f.scene,
        layer: f.layer,
        property: celframe_core::LayerProperty::Opacity,
        key: opacity_key(frame, value),
    };
    add(3, 0.5).apply(&mut project).unwrap();
    assert_reversible(project.clone(), add(3, 0.9));
    assert_reversible(
        project,
        Command::RemoveKey {
            scene: f.scene,
            layer: f.layer,
            property: celframe_core::LayerProperty::Opacity,
            frame: 3,
        },
    );
}

#[test]
fn opacity_outside_0_to_1_is_refused() {
    let f = fixture();
    assert_rejected(
        f.project.clone(),
        Command::SetBaseValue {
            scene: f.scene,
            layer: f.layer,
            property: celframe_core::LayerProperty::Opacity,
            value: 1.5,
        },
        CommandError::InvalidValue,
    );
    assert_rejected(
        f.project,
        Command::SetKey {
            scene: f.scene,
            layer: f.layer,
            property: celframe_core::LayerProperty::Opacity,
            key: opacity_key(0, f64::NAN),
        },
        CommandError::InvalidValue,
    );
}
