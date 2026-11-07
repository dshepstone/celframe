use celframe_core::{
    Command, Drawing, DrawingId, History, LayerId, Project, SceneId, Stroke, StrokeId, StrokeSample,
};

struct Doc {
    project: Project,
    scene: SceneId,
    layer: LayerId,
    drawing: DrawingId,
}

fn doc() -> Doc {
    let mut project = Project::new("A");
    let scene = project.scenes[0].id;
    let layer = project.scenes[0].layers[0].id;
    let drawing = Drawing::new();
    let drawing_id = drawing.id;
    project.scenes[0].layers[0].drawings.push(drawing);
    Doc {
        project,
        scene,
        layer,
        drawing: drawing_id,
    }
}

fn sample(x: f64) -> StrokeSample {
    StrokeSample {
        x,
        y: 0.0,
        pressure: 1.0,
    }
}

impl Doc {
    fn add(&self, stroke: &Stroke) -> Command {
        Command::AddStroke {
            scene: self.scene,
            layer: self.layer,
            drawing: self.drawing,
            stroke: stroke.clone(),
            index: 0,
        }
    }

    fn append(&self, stroke: StrokeId, xs: &[f64]) -> Command {
        Command::AppendStrokeSamples {
            scene: self.scene,
            layer: self.layer,
            drawing: self.drawing,
            stroke,
            samples: xs.iter().copied().map(sample).collect(),
        }
    }

    fn strokes(&self) -> &[Stroke] {
        &self.project.scenes[0].layers[0].drawings[0].strokes
    }
}

fn new_stroke() -> Stroke {
    Stroke {
        id: StrokeId::new(),
        base_width: 4.0,
        color: [0, 0, 0, 255],
        art: celframe_core::Art::Line,
        samples: vec![sample(0.0)],
    }
}

#[test]
fn a_continuous_stroke_undoes_and_redoes_as_one_step() {
    let mut d = doc();
    let mut history = History::new();
    let stroke = new_stroke();
    let add = d.add(&stroke);
    history.begin_stroke(&mut d.project, add).unwrap();
    for x in [1.0, 2.0, 3.0] {
        let append = d.append(stroke.id, &[x]);
        history.execute(&mut d.project, append).unwrap();
    }
    history.end_stroke();
    let finished = d.strokes().to_vec();
    assert_eq!(finished[0].samples.len(), 4);

    assert!(history.undo(&mut d.project).unwrap());
    assert!(d.strokes().is_empty());
    assert!(!history.can_undo());

    assert!(history.redo(&mut d.project).unwrap());
    assert_eq!(d.strokes(), finished);
}

#[test]
fn a_finished_stroke_does_not_merge_with_later_appends() {
    let mut d = doc();
    let mut history = History::new();
    let stroke = new_stroke();
    let add = d.add(&stroke);
    history.begin_stroke(&mut d.project, add).unwrap();
    history.end_stroke();
    let append = d.append(stroke.id, &[1.0]);
    history.execute(&mut d.project, append).unwrap();

    history.undo(&mut d.project).unwrap();
    assert_eq!(d.strokes()[0].samples.len(), 1);
    history.undo(&mut d.project).unwrap();
    assert!(d.strokes().is_empty());
}

#[test]
fn appends_to_a_different_stroke_do_not_merge() {
    let mut d = doc();
    let mut history = History::new();
    let first = new_stroke();
    let add = d.add(&first);
    history.execute(&mut d.project, add).unwrap();
    let second = new_stroke();
    let add = d.add(&second);
    history.begin_stroke(&mut d.project, add).unwrap();
    let append = d.append(first.id, &[1.0]);
    history.execute(&mut d.project, append).unwrap();

    history.undo(&mut d.project).unwrap();
    assert_eq!(d.strokes().len(), 2);
    let first_now = d.strokes().iter().find(|s| s.id == first.id).unwrap();
    assert_eq!(first_now.samples.len(), 1);
}

#[test]
fn undo_closes_an_open_stroke() {
    let mut d = doc();
    let mut history = History::new();
    let stroke = new_stroke();
    let add = d.add(&new_stroke());
    history.execute(&mut d.project, add).unwrap();
    let add = d.add(&stroke);
    history.begin_stroke(&mut d.project, add).unwrap();
    history.undo(&mut d.project).unwrap();
    history.redo(&mut d.project).unwrap();
    let append = d.append(stroke.id, &[1.0]);
    history.execute(&mut d.project, append).unwrap();
    history.undo(&mut d.project).unwrap();
    let restored = d.strokes().iter().find(|s| s.id == stroke.id).unwrap();
    assert_eq!(restored.samples.len(), 1);
}

#[test]
fn merging_keeps_dirty_state_correct_and_clears_redo() {
    let mut d = doc();
    let mut history = History::new();
    let old = new_stroke();
    let add = d.add(&old);
    history.execute(&mut d.project, add).unwrap();
    history.undo(&mut d.project).unwrap();
    assert!(history.can_redo());

    let stroke = new_stroke();
    let add = d.add(&stroke);
    history.begin_stroke(&mut d.project, add).unwrap();
    assert!(!history.can_redo());
    history.mark_saved();
    let append = d.append(stroke.id, &[1.0]);
    history.execute(&mut d.project, append).unwrap();
    assert!(
        history.is_dirty(),
        "a merged append is still an unsaved edit"
    );
}

#[test]
fn append_and_truncate_reverse_each_other() {
    let mut d = doc();
    let stroke = new_stroke();
    d.add(&stroke).apply(&mut d.project).unwrap();
    let before = d.project.clone();
    let inverse = d
        .append(stroke.id, &[1.0, 2.0])
        .apply(&mut d.project)
        .unwrap();
    let after = d.project.clone();
    let redo = inverse.apply(&mut d.project).unwrap();
    assert_eq!(d.project, before);
    redo.apply(&mut d.project).unwrap();
    assert_eq!(d.project, after);
}
