//! The open document, its history and its file. Tauri handlers only wrap these methods.

use std::path::{Path, PathBuf};

use celframe_core::{
    Command, CommandError, Drawing, DrawingId, Exposure, History, LayerId, Project, SceneId,
    Stroke, StrokeId, StrokeSample,
};
use celframe_geom::{Constraints, Shape, shape_samples};

use crate::eraser::EraserDrag;
use crate::pen::{self, PenOptions, PenStroke, PenTool};
use crate::select::{self, SelectDrag, Selection};
use serde::Serialize;

pub struct Session {
    project: Project,
    history: History,
    path: Option<PathBuf>,
    autosave: AutosaveState,
    /// A recovery copy from an earlier run is waiting for the person to restore or discard it.
    recovery_pending: bool,
    /// The stroke being drawn, if the pen is down.
    pen: Option<PenStroke>,
    /// The eraser drag in progress, if the pen is down and erasing.
    eraser: Option<EraserDrag>,
    /// The shape being dragged out, if the pen is down with the Shape tool.
    shape: Option<ShapeDrag>,
    /// The selected strokes. Not part of the document.
    selection: Option<Selection>,
    /// The Select tool's drag in progress.
    select: Option<SelectDrag>,
    /// The modifier keys held at the last pointer event.
    modifiers: Constraints,
}

/// A shape being dragged out. Its stroke is added on the first movement that gives it a size.
struct ShapeDrag {
    scene: SceneId,
    layer: LayerId,
    /// The drawing shown at frame 0. None until the shape's stroke creates one.
    drawing: Option<DrawingId>,
    stroke: Option<StrokeId>,
    shape: Shape,
    anchor: [f64; 2],
    last: [f64; 2],
    template: Stroke,
}

/// What the recovery copy on disk holds, as far as this session knows.
#[derive(Clone, Copy, Debug, PartialEq)]
enum AutosaveState {
    /// No copy written by this session.
    None,
    /// A copy of this document at this history revision.
    Written(u64),
    /// A copy that is out of date or belongs to a replaced document.
    Stale,
}

/// What the autosave timer should do next.
#[derive(Debug, PartialEq)]
pub enum AutosaveStep {
    Nothing,
    /// Write this project as the recovery copy. The path is its own file, if any.
    Write(Project, Option<PathBuf>),
    /// Remove the recovery copy.
    Clear,
}

/// What the UI renders. It is always replaced as a whole, never edited in the UI.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentView {
    pub project: Project,
    pub can_undo: bool,
    pub can_redo: bool,
    pub dirty: bool,
    /// The file the project was opened from or last saved to.
    pub path: Option<PathBuf>,
    /// The selected strokes. See `docs/specs/selection.md`.
    pub selection: Vec<StrokeId>,
}

impl Session {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            history: History::new(),
            path: None,
            autosave: AutosaveState::None,
            recovery_pending: false,
            pen: None,
            eraser: None,
            shape: None,
            selection: None,
            select: None,
            modifiers: Constraints::default(),
        }
    }

    /// Swaps in another project with a fresh history, keeping track of the recovery copy.
    fn replace(&mut self, project: Project, path: Option<PathBuf>) {
        self.project = project;
        self.history = History::new();
        self.path = path;
        self.end_drags();
        self.selection = None;
        if self.autosave != AutosaveState::None {
            self.autosave = AutosaveState::Stale;
        }
    }

    pub fn view(&self) -> DocumentView {
        DocumentView {
            project: self.project.clone(),
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
            dirty: self.history.is_dirty(),
            path: self.path.clone(),
            selection: self.selected_ids(),
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    pub fn project_name(&self) -> &str {
        &self.project.name
    }

    pub fn is_dirty(&self) -> bool {
        self.history.is_dirty()
    }

    /// Adds an empty layer on top of the first scene, with an ID the core chooses.
    pub fn add_layer(&mut self) -> Result<DocumentView, CommandError> {
        let scene = self.project.scenes.first().ok_or(CommandError::NotFound)?;
        let name = format!("Layer {}", scene.layers.len() + 1);
        let command = Command::AddLayer {
            scene: scene.id,
            layer: celframe_core::Layer::new(name),
            index: scene.layers.len(),
        };
        self.execute(command)
    }

    pub fn execute(&mut self, command: Command) -> Result<DocumentView, CommandError> {
        // Any other edit ends a drag in progress, as it does in the history.
        self.end_drags();
        self.history.execute(&mut self.project, command)?;
        Ok(self.view())
    }

    pub fn undo(&mut self) -> Result<DocumentView, CommandError> {
        self.end_drags();
        self.history.undo(&mut self.project)?;
        Ok(self.view())
    }

    pub fn redo(&mut self) -> Result<DocumentView, CommandError> {
        self.end_drags();
        self.history.redo(&mut self.project)?;
        Ok(self.view())
    }

    /// Replaces the project with an empty, unsaved one.
    pub fn new_document(&mut self) -> DocumentView {
        self.replace(Project::starter("Untitled"), None);
        self.view()
    }

    /// Opens a file. On failure the current project stays open, unchanged.
    pub fn open(&mut self, path: &Path) -> Result<DocumentView, String> {
        let project = celframe_format::load_project(path)
            .map_err(|error| format!("Couldn't open {}: {error}", display_name(path)))?;
        self.replace(project, Some(path.to_owned()));
        Ok(self.view())
    }

    /// Saves to exactly `path`, which becomes the project's file. History is kept.
    pub fn save_to(&mut self, path: &Path) -> Result<DocumentView, String> {
        celframe_format::save_project(&self.project, path)
            .map_err(|error| format!("Couldn't save {}: {error}", display_name(path)))?;
        self.history.mark_saved();
        self.path = Some(path.to_owned());
        Ok(self.view())
    }

    /// Saves to a path chosen in Save As, adding `.celf` if it has no extension.
    pub fn save_as(&mut self, chosen: &Path) -> Result<DocumentView, String> {
        let path = with_celf_extension(chosen);
        // The picker only confirmed replacing `chosen`, not a file we named for it.
        if path != chosen && path.exists() {
            return Err(format!(
                "Couldn't save {}: a file with that name already exists. Choose it in Save As to replace it.",
                display_name(&path)
            ));
        }
        self.save_to(&path)
    }
}

impl Session {
    pub fn set_recovery_pending(&mut self, pending: bool) {
        self.recovery_pending = pending;
    }

    pub fn recovery_pending(&self) -> bool {
        self.recovery_pending
    }

    /// Opens restored work as unsaved changes to its original file, if it had one.
    pub fn restore(&mut self, project: Project, original_path: Option<PathBuf>) -> DocumentView {
        self.replace(project, original_path);
        self.history.mark_unsaved();
        // The copy on disk is the one being restored, so rewrite it at the next step.
        self.autosave = AutosaveState::Stale;
        self.recovery_pending = false;
        self.view()
    }

    /// Decides what autosave does now. Call `autosave_failed` if a write fails.
    pub fn autosave_step(&mut self) -> AutosaveStep {
        if self.recovery_pending {
            return AutosaveStep::Nothing;
        }
        let revision = self.history.revision();
        match (self.history.is_dirty(), self.autosave) {
            (true, AutosaveState::Written(written)) if written == revision => AutosaveStep::Nothing,
            (true, _) => {
                self.autosave = AutosaveState::Written(revision);
                AutosaveStep::Write(self.project.clone(), self.path.clone())
            }
            (false, AutosaveState::None) => AutosaveStep::Nothing,
            (false, _) => {
                self.autosave = AutosaveState::None;
                AutosaveStep::Clear
            }
        }
    }

    pub fn autosave_failed(&mut self) {
        self.autosave = AutosaveState::Stale;
    }

    /// True when exiting should remove the recovery copy.
    pub fn should_clear_recovery_on_exit(&self) -> bool {
        !self.recovery_pending
    }
}

impl Session {
    /// Starts a stroke at a scene-space sample. `min_distance` is in scene units.
    pub fn pen_down(
        &mut self,
        sample: StrokeSample,
        min_distance: f64,
        options: PenOptions,
    ) -> Result<(), String> {
        if self.pen.is_some()
            || self.eraser.is_some()
            || self.shape.is_some()
            || self.select.is_some()
        {
            self.pen_up()?;
        }
        if !pen::PEN_WIDTH_RANGE.contains(&options.width) {
            return Err("That brush size isn't valid.".to_owned());
        }
        let first = pen::clean(sample).ok_or("That pen position isn't valid.")?;
        let (scene, layer, drawing) = pen::find_target(&self.project, 0, options.layer)?;
        if options.tool == PenTool::Select {
            // 8 half-pixels is 4 device pixels.
            let target = (scene, layer, drawing, options.art);
            return self.select_down([first.x, first.y], 8.0 * min_distance, target);
        }
        if let PenTool::Erase { whole_strokes } = options.tool {
            self.history.begin_group();
            self.eraser = Some(EraserDrag {
                scene,
                layer,
                drawing,
                art: options.art,
                radius: options.width / 2.0,
                whole_strokes,
                last: [first.x, first.y],
            });
            return self.erase(&[first]);
        }
        if let PenTool::Shape { shape } = options.tool {
            let shape = Shape::from(shape);
            if let Shape::Polygon { sides } = shape
                && !celframe_geom::POLYGON_SIDES.contains(&sides)
            {
                return Err("A polygon needs 3 to 12 sides.".to_owned());
            }
            let point = [first.x, first.y];
            self.shape = Some(ShapeDrag {
                scene,
                layer,
                drawing,
                stroke: None,
                shape,
                anchor: point,
                last: point,
                template: Stroke {
                    id: StrokeId::new(),
                    base_width: options.width,
                    color: options.color,
                    art: options.art,
                    samples: Vec::new(),
                },
            });
            return Ok(());
        }
        let mut pen = None;
        self.begin_stroke(scene, layer, drawing, |drawing| {
            let (started, stroke) =
                PenStroke::start((scene, layer, drawing), first, min_distance, options);
            pen = Some(started);
            stroke
        })?;
        self.pen = pen;
        Ok(())
    }

    /// Sets which modifier keys are held. Shapes use them from the next movement, and the
    /// Select tool when the pen goes down.
    pub fn set_modifiers(&mut self, modifiers: Constraints) {
        self.modifiers = modifiers;
    }

    /// Redraws the dragged shape to end at its last point, as part of its one undo step.
    fn reshape(&mut self) -> Result<(), String> {
        let Some(drag) = self.shape.as_ref() else {
            return Ok(());
        };
        let Some(samples) = shape_samples(drag.shape, drag.anchor, drag.last, self.modifiers)
        else {
            return Ok(());
        };
        let (scene, layer) = (drag.scene, drag.layer);
        let result = match (drag.drawing, drag.stroke) {
            (Some(drawing), Some(stroke)) => [
                Command::TruncateStrokeSamples {
                    scene,
                    layer,
                    drawing,
                    stroke,
                    len: 0,
                },
                Command::AppendStrokeSamples {
                    scene,
                    layer,
                    drawing,
                    stroke,
                    samples,
                },
            ]
            .into_iter()
            .try_for_each(|command| {
                self.history
                    .execute(&mut self.project, command)
                    .map_err(|e| e.to_string())
            }),
            _ => {
                let stroke = Stroke {
                    samples,
                    ..drag.template.clone()
                };
                let id = stroke.id;
                self.begin_stroke(scene, layer, drag.drawing, |_| stroke)
                    .map(|drawing| {
                        let drag = self.shape.as_mut().expect("the drag is still open");
                        drag.drawing = Some(drawing);
                        drag.stroke = Some(id);
                    })
            }
        };
        if result.is_err() {
            self.end_drags();
        }
        result
    }

    /// Adds a stroke as one undo step that later appends and reshapes merge into. When no
    /// drawing is shown at frame 0, a new one is added in the same step.
    fn begin_stroke(
        &mut self,
        scene: SceneId,
        layer: LayerId,
        drawing: Option<DrawingId>,
        stroke: impl FnOnce(DrawingId) -> Stroke,
    ) -> Result<DrawingId, String> {
        let target = self
            .project
            .scenes
            .iter()
            .find(|s| s.id == scene)
            .and_then(|s| s.layers.iter().find(|l| l.id == layer))
            .ok_or("That layer no longer exists.")?;
        let mut commands = Vec::new();
        let (drawing, index) = match drawing {
            Some(id) => {
                let index = target
                    .drawings
                    .iter()
                    .find(|d| d.id == id)
                    .map_or(0, |d| d.strokes.len());
                (id, index)
            }
            None => {
                let new = Drawing::new();
                let id = new.id;
                commands.push(Command::AddDrawing {
                    scene,
                    layer,
                    drawing: new,
                    index: target.drawings.len(),
                });
                commands.push(Command::InsertExposure {
                    scene,
                    layer,
                    exposure: Exposure::new(0, 1, id),
                });
                (id, 0)
            }
        };
        commands.push(Command::AddStroke {
            scene,
            layer,
            drawing,
            stroke: stroke(drawing),
            index,
        });
        let command = if commands.len() == 1 {
            commands.pop().expect("one command")
        } else {
            Command::Batch { commands }
        };
        self.history
            .begin_stroke(&mut self.project, command)
            .map_err(|e| e.to_string())?;
        Ok(drawing)
    }

    /// Adds scene-space samples to the stroke in progress. Ignored when the pen isn't down.
    pub fn pen_move(&mut self, samples: &[StrokeSample]) -> Result<(), String> {
        if self.eraser.is_some() {
            let samples: Vec<_> = samples.iter().copied().filter_map(pen::clean).collect();
            return self.erase(&samples);
        }
        if self.select.is_some() {
            return self.select_move(samples);
        }
        if let Some(drag) = self.shape.as_mut() {
            if let Some(last) = samples.iter().rev().copied().find_map(pen::clean) {
                drag.last = [last.x, last.y];
            }
            return self.reshape();
        }
        let Some(pen) = self.pen.as_mut() else {
            return Ok(());
        };
        let samples = pen.push(samples);
        self.append(samples)
    }

    /// Ends the stroke or eraser drag in progress, if any.
    pub fn pen_up(&mut self) -> Result<DocumentView, String> {
        if self.eraser.take().is_some() {
            self.history.end_group();
        }
        if self.shape.take().is_some() {
            self.history.end_stroke();
        }
        if self.select.is_some() {
            self.select_up()?;
        }
        if let Some(pen) = self.pen.take() {
            let target = (pen.scene, pen.layer, pen.drawing, pen.stroke);
            let samples = pen.finish();
            self.append_to(target, samples)?;
            self.history.end_stroke();
        }
        Ok(self.view())
    }

    fn end_drags(&mut self) {
        self.pen = None;
        self.shape = None;
        self.cancel_select();
        if self.eraser.take().is_some() {
            self.history.end_group();
        }
    }

    /// Erases along the eraser's path to `samples`, merging into the drag's undo step.
    fn erase(&mut self, samples: &[StrokeSample]) -> Result<(), String> {
        let Some(eraser) = self.eraser.as_mut() else {
            return Ok(());
        };
        let commands = eraser.erase_to(&self.project, samples);
        if commands.is_empty() {
            return Ok(());
        }
        let result = self
            .history
            .execute(&mut self.project, Command::Batch { commands });
        if result.is_err() {
            self.end_drags();
        }
        result.map_err(|e| e.to_string())
    }

    fn append(&mut self, samples: Vec<StrokeSample>) -> Result<(), String> {
        let Some(pen) = self.pen.as_ref() else {
            return Ok(());
        };
        let target = (pen.scene, pen.layer, pen.drawing, pen.stroke);
        let result = self.append_to(target, samples);
        if result.is_err() {
            self.pen = None;
        }
        result
    }

    fn append_to(
        &mut self,
        (scene, layer, drawing, stroke): (
            celframe_core::SceneId,
            celframe_core::LayerId,
            celframe_core::DrawingId,
            celframe_core::StrokeId,
        ),
        samples: Vec<StrokeSample>,
    ) -> Result<(), String> {
        if samples.is_empty() {
            return Ok(());
        }
        let command = Command::AppendStrokeSamples {
            scene,
            layer,
            drawing,
            stroke,
            samples,
        };
        // The history merges this into the stroke's own undo step.
        self.history
            .execute(&mut self.project, command)
            .map_err(|e| e.to_string())
    }
}

impl Session {
    fn selected_ids(&self) -> Vec<StrokeId> {
        self.selection.as_ref().map_or_else(Vec::new, |s| {
            s.existing(&self.project).iter().map(|s| s.id).collect()
        })
    }

    /// Outlines for the selection and any box being dragged out. `pixel` is one device pixel
    /// in scene units.
    pub fn overlay(&self, pixel: f64) -> Vec<Stroke> {
        let selection = self
            .selection
            .as_ref()
            .and_then(|s| s.bounds(&self.project));
        let marquee = match &self.select {
            Some(SelectDrag::Marquee { start, last, .. }) if start != last => Some([
                start[0].min(last[0]),
                start[1].min(last[1]),
                start[0].max(last[0]),
                start[1].max(last[1]),
            ]),
            _ => None,
        };
        select::overlay(selection, marquee, pixel)
    }

    fn select_down(
        &mut self,
        point: [f64; 2],
        tolerance: f64,
        (scene, layer, drawing, art): (SceneId, LayerId, Option<DrawingId>, celframe_core::Art),
    ) -> Result<(), String> {
        let Some(drawing) = drawing else {
            self.selection = None;
            return Ok(());
        };
        let target = select::drawing(&self.project, scene, layer, drawing)
            .ok_or("That drawing no longer exists.")?;
        let same_place = self
            .selection
            .as_ref()
            .is_some_and(|s| (s.scene, s.layer, s.drawing, s.art) == (scene, layer, drawing, art));
        if !same_place {
            self.selection = Some(Selection {
                scene,
                layer,
                drawing,
                art,
                strokes: Vec::new(),
            });
        }
        let selection = self.selection.as_mut().expect("set above");
        selection
            .strokes
            .retain(|id| target.strokes.iter().any(|s| s.id == *id));
        let hit = select::hit(target, art, point, tolerance);

        if self.modifiers.constrain {
            match hit {
                Some(id) => {
                    if let Some(i) = selection.strokes.iter().position(|s| *s == id) {
                        selection.strokes.remove(i);
                    } else {
                        selection.strokes.push(id);
                    }
                }
                None => {
                    self.select = Some(SelectDrag::Marquee {
                        start: point,
                        last: point,
                        add: true,
                    });
                }
            }
            return Ok(());
        }

        let inside = selection
            .bounds(&self.project)
            .is_some_and(|area| select::contains(area, point));
        let click = match hit {
            Some(id) if selection.strokes.contains(&id) => Some(vec![id]),
            Some(id) => {
                selection.strokes = vec![id];
                None
            }
            None if inside => Some(Vec::new()),
            None => {
                selection.strokes.clear();
                self.select = Some(SelectDrag::Marquee {
                    start: point,
                    last: point,
                    add: false,
                });
                return Ok(());
            }
        };
        let originals = selection.originals(&self.project);
        self.select = Some(SelectDrag::Move {
            start: point,
            last: point,
            originals,
            click,
        });
        Ok(())
    }

    fn select_move(&mut self, samples: &[StrokeSample]) -> Result<(), String> {
        let Some(point) = samples.iter().rev().copied().find_map(pen::clean) else {
            return Ok(());
        };
        let point = [point.x, point.y];
        match self.select.as_mut() {
            Some(SelectDrag::Marquee { last, .. }) => *last = point,
            Some(SelectDrag::Move {
                start,
                last,
                originals,
                ..
            }) => {
                *last = point;
                let delta = [point[0] - start[0], point[1] - start[1]];
                let Some(selection) = self.selection.as_ref() else {
                    return Ok(());
                };
                // A preview only: the move becomes one undoable edit when the pen lifts.
                if selection
                    .moved(originals, delta)
                    .apply(&mut self.project)
                    .is_err()
                {
                    self.cancel_select();
                    return Err("The selection changed while moving it.".to_owned());
                }
            }
            None => {}
        }
        Ok(())
    }

    fn select_up(&mut self) -> Result<(), String> {
        match self.select.take() {
            Some(SelectDrag::Marquee { start, last, add }) => {
                let Some(selection) = self.selection.as_mut() else {
                    return Ok(());
                };
                if start == last {
                    return Ok(());
                }
                let found = select::drawing(
                    &self.project,
                    selection.scene,
                    selection.layer,
                    selection.drawing,
                )
                .map_or_else(Vec::new, |d| select::in_box(d, selection.art, start, last));
                if !add {
                    selection.strokes.clear();
                }
                for id in found {
                    if !selection.strokes.contains(&id) {
                        selection.strokes.push(id);
                    }
                }
                Ok(())
            }
            Some(SelectDrag::Move {
                start,
                last,
                originals,
                click,
            }) => {
                let Some(selection) = self.selection.clone() else {
                    return Ok(());
                };
                let delta = [last[0] - start[0], last[1] - start[1]];
                // Back to where the drag started, then the move as one edit.
                let _ = selection
                    .moved(&originals, [0.0, 0.0])
                    .apply(&mut self.project);
                if delta == [0.0, 0.0] {
                    if let (Some(click), Some(selection)) = (click, self.selection.as_mut()) {
                        selection.strokes = click;
                    }
                    return Ok(());
                }
                self.history
                    .execute(&mut self.project, selection.moved(&originals, delta))
                    .map_err(|e| e.to_string())
            }
            None => Ok(()),
        }
    }

    /// Ends a Select drag without changing anything, undoing a move's preview.
    fn cancel_select(&mut self) {
        if let Some(SelectDrag::Move { originals, .. }) = self.select.take()
            && let Some(selection) = self.selection.as_ref()
        {
            let _ = selection
                .moved(&originals, [0.0, 0.0])
                .apply(&mut self.project);
        }
    }

    /// Selects every stroke in a part of the current layer's drawing at frame 0.
    pub fn select_all(
        &mut self,
        layer: Option<LayerId>,
        art: celframe_core::Art,
    ) -> Result<DocumentView, String> {
        self.end_drags();
        let (scene, layer, drawing) = pen::find_target(&self.project, 0, layer)?;
        self.selection = drawing.and_then(|drawing| {
            let strokes = select::drawing(&self.project, scene, layer, drawing)?
                .strokes
                .iter()
                .filter(|s| s.art == art)
                .map(|s| s.id)
                .collect();
            Some(Selection {
                scene,
                layer,
                drawing,
                art,
                strokes,
            })
        });
        Ok(self.view())
    }

    pub fn clear_selection(&mut self) -> DocumentView {
        self.end_drags();
        self.selection = None;
        self.view()
    }

    /// Deletes the selected strokes as one undo step.
    pub fn delete_selection(&mut self) -> Result<DocumentView, String> {
        self.end_drags();
        let Some(selection) = self.selection.take() else {
            return Ok(self.view());
        };
        let commands: Vec<Command> = selection
            .existing(&self.project)
            .iter()
            .map(|s| Command::RemoveStroke {
                scene: selection.scene,
                layer: selection.layer,
                drawing: selection.drawing,
                stroke: s.id,
            })
            .collect();
        if !commands.is_empty() {
            self.history
                .execute(&mut self.project, Command::Batch { commands })
                .map_err(|e| e.to_string())?;
        }
        Ok(self.view())
    }

    /// Moves the selected strokes by `delta` scene units as one undo step.
    pub fn nudge_selection(&mut self, delta: [f64; 2]) -> Result<DocumentView, String> {
        self.end_drags();
        if !delta.iter().all(|d| d.is_finite()) {
            return Err("That move isn't valid.".to_owned());
        }
        if let Some(selection) = self.selection.as_ref() {
            let originals = selection.originals(&self.project);
            if !originals.is_empty() {
                let command = selection.moved(&originals, delta);
                self.history
                    .execute(&mut self.project, command)
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(self.view())
    }
}

/// Adds `.celf` to a path with no extension.
fn with_celf_extension(path: &Path) -> PathBuf {
    if path.extension().is_some() {
        path.to_owned()
    } else {
        path.with_extension("celf")
    }
}

fn display_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rename(name: &str) -> Command {
        Command::RenameProject {
            name: name.to_owned(),
        }
    }

    #[test]
    fn views_track_edits_undo_and_redo() {
        let mut session = Session::new(Project::new("A"));
        let view = session.view();
        assert!(!view.can_undo && !view.can_redo && !view.dirty);

        let view = session.execute(rename("B")).unwrap();
        assert_eq!(view.project.name, "B");
        assert!(view.can_undo && view.dirty);

        let view = session.undo().unwrap();
        assert_eq!(view.project.name, "A");
        assert!(view.can_redo && !view.dirty);

        let view = session.redo().unwrap();
        assert_eq!(view.project.name, "B");
    }

    #[test]
    fn views_serialize_for_the_ui_in_camel_case() {
        let json = serde_json::to_value(Session::new(Project::new("A")).view()).unwrap();
        assert_eq!(json["canUndo"], false);
        assert_eq!(json["project"]["name"], "A");
        assert!(json["path"].is_null());
    }

    #[test]
    fn new_document_discards_the_project_history_and_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(Project::new("A"));
        session.save_to(&dir.path().join("a.celf")).unwrap();
        session.execute(rename("B")).unwrap();

        let view = session.new_document();
        assert_eq!(view.project.name, "Untitled");
        assert!(!view.can_undo && !view.dirty && view.path.is_none());
    }

    #[test]
    fn save_marks_clean_keeps_history_and_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ball.celf");
        let mut session = Session::new(Project::new("A"));
        session.execute(rename("Ball")).unwrap();

        let view = session.save_to(&path).unwrap();
        assert!(!view.dirty && view.can_undo);
        assert_eq!(view.path.as_deref(), Some(path.as_path()));

        // Undoing past the save makes the project dirty again.
        assert!(session.undo().unwrap().dirty);

        let mut other = Session::new(Project::new("Other"));
        let opened = other.open(&path).unwrap();
        assert_eq!(opened.project, view.project);
        assert!(!opened.dirty && !opened.can_undo);
        assert_eq!(opened.path.as_deref(), Some(path.as_path()));
    }

    #[test]
    fn save_as_adds_the_extension_only_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(Project::new("A"));

        let view = session.save_as(&dir.path().join("plain")).unwrap();
        assert_eq!(view.path, Some(dir.path().join("plain.celf")));
        assert!(dir.path().join("plain.celf").is_file());

        let view = session.save_as(&dir.path().join("kept.backup")).unwrap();
        assert_eq!(view.path, Some(dir.path().join("kept.backup")));
    }

    #[test]
    fn save_as_never_replaces_a_file_the_picker_did_not_confirm() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("plain.celf");
        std::fs::write(&existing, b"someone else's file").unwrap();
        let mut session = Session::new(Project::new("A"));
        session.execute(rename("B")).unwrap();

        let error = session.save_as(&dir.path().join("plain")).unwrap_err();
        assert!(error.contains("plain.celf"), "{error}");
        assert_eq!(std::fs::read(&existing).unwrap(), b"someone else's file");
        assert!(session.view().dirty);

        // Choosing the existing file itself is a confirmed replace.
        assert!(!session.save_as(&existing).unwrap().dirty);
    }

    #[test]
    fn save_keeps_the_exact_path_of_a_file_with_no_extension() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no-extension");
        let mut session = Session::new(Project::new("A"));
        session.save_to(&path).unwrap();
        session.open(&path).unwrap();
        session.execute(rename("B")).unwrap();

        let view = session.save_to(&path).unwrap();
        assert_eq!(view.path, Some(path.clone()));
        assert!(!dir.path().join("no-extension.celf").exists());
        let mut other = Session::new(Project::new("Other"));
        assert_eq!(other.open(&path).unwrap().project.name, "B");
    }

    #[test]
    fn failed_open_leaves_the_project_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("broken.celf");
        std::fs::write(&bad, b"not a zip").unwrap();
        let mut session = Session::new(Project::new("A"));
        session.execute(rename("Edited")).unwrap();
        let before = session.view();

        let error = session.open(&bad).unwrap_err();
        assert!(error.starts_with("Couldn't open broken.celf: "), "{error}");
        assert_eq!(session.view(), before);
    }

    fn write_step(session: &mut Session) -> Project {
        match session.autosave_step() {
            AutosaveStep::Write(project, _) => project,
            other => panic!("expected a write, got {other:?}"),
        }
    }

    #[test]
    fn autosave_writes_each_new_revision_once_and_clears_when_clean() {
        let dir = tempfile::tempdir().unwrap();
        let mut session = Session::new(Project::new("A"));
        assert_eq!(session.autosave_step(), AutosaveStep::Nothing);

        session.execute(rename("B")).unwrap();
        assert_eq!(write_step(&mut session).name, "B");
        assert_eq!(session.autosave_step(), AutosaveStep::Nothing);

        session.execute(rename("C")).unwrap();
        assert_eq!(write_step(&mut session).name, "C");

        session.save_to(&dir.path().join("a.celf")).unwrap();
        assert_eq!(session.autosave_step(), AutosaveStep::Clear);
        assert_eq!(session.autosave_step(), AutosaveStep::Nothing);

        // Undoing past the save is unsaved work again.
        session.undo().unwrap();
        assert_eq!(write_step(&mut session).name, "B");
    }

    #[test]
    fn autosave_retries_after_a_failed_write() {
        let mut session = Session::new(Project::new("A"));
        session.execute(rename("B")).unwrap();
        write_step(&mut session);
        session.autosave_failed();
        assert_eq!(write_step(&mut session).name, "B");
    }

    #[test]
    fn replacing_the_document_clears_its_copy_even_at_a_matching_revision() {
        let mut session = Session::new(Project::new("A"));
        session.execute(rename("B")).unwrap();
        write_step(&mut session);

        session.new_document();
        assert_eq!(session.autosave_step(), AutosaveStep::Clear);

        // A new document's revisions start again, so they must not match the old copy.
        session.execute(rename("Fresh")).unwrap();
        assert_eq!(write_step(&mut session).name, "Fresh");
    }

    #[test]
    fn a_pending_recovery_blocks_autosave_until_restored() {
        let mut session = Session::new(Project::new("Untitled"));
        session.set_recovery_pending(true);
        session.execute(rename("B")).unwrap();
        assert_eq!(session.autosave_step(), AutosaveStep::Nothing);
        assert!(!session.should_clear_recovery_on_exit());

        let view = session.restore(Project::new("Recovered"), Some("ball.celf".into()));
        assert!(view.dirty && !view.can_undo);
        assert_eq!(view.path, Some(PathBuf::from("ball.celf")));
        assert!(session.should_clear_recovery_on_exit());
        match session.autosave_step() {
            AutosaveStep::Write(project, path) => {
                assert_eq!(project.name, "Recovered");
                assert_eq!(path, Some(PathBuf::from("ball.celf")));
            }
            other => panic!("expected a write, got {other:?}"),
        }
    }

    fn sample(x: f64, y: f64) -> StrokeSample {
        StrokeSample {
            x,
            y,
            pressure: 0.5,
        }
    }

    fn strokes_at_frame_0(session: &Session) -> Vec<celframe_core::Stroke> {
        session.project().scenes[0]
            .visible_strokes(0)
            .cloned()
            .collect()
    }

    fn draw(session: &mut Session, points: &[(f64, f64)]) {
        session
            .pen_down(sample(points[0].0, points[0].1), 0.1, PenOptions::default())
            .unwrap();
        let rest: Vec<_> = points[1..].iter().map(|&(x, y)| sample(x, y)).collect();
        session.pen_move(&rest).unwrap();
        session.pen_up().unwrap();
    }

    #[test]
    fn a_pen_stroke_is_smoothed_and_undoes_in_one_step() {
        let mut session = Session::new(Project::starter("A"));
        let points: Vec<_> = (0..20)
            .map(|i| (f64::from(i) * 5.0, if i % 2 == 0 { 1.0 } else { -1.0 }))
            .collect();
        draw(&mut session, &points);

        let strokes = strokes_at_frame_0(&session);
        assert_eq!(strokes.len(), 1);
        let stroke = &strokes[0];
        assert_eq!(stroke.samples.len(), 20);
        // The ends are kept exactly; the zigzag in between is flattened.
        assert_eq!(stroke.samples[0], sample(0.0, 1.0));
        assert_eq!(stroke.samples[19], sample(95.0, -1.0));
        assert!(stroke.samples[10].y.abs() < 1.0);
        assert_eq!(stroke.base_width, pen::PEN_WIDTH);

        let view = session.view();
        assert!(view.dirty && view.can_undo);
        session.undo().unwrap();
        assert!(strokes_at_frame_0(&session).is_empty());
        assert!(!session.view().can_undo);
    }

    #[test]
    fn a_tap_is_a_one_sample_stroke() {
        let mut session = Session::new(Project::starter("A"));
        draw(&mut session, &[(3.0, 4.0)]);
        assert_eq!(strokes_at_frame_0(&session)[0].samples, [sample(3.0, 4.0)]);
    }

    #[test]
    fn samples_closer_than_the_minimum_distance_are_dropped() {
        let mut session = Session::new(Project::starter("A"));
        draw(
            &mut session,
            &[(0.0, 0.0), (0.01, 0.0), (0.02, 0.0), (10.0, 0.0)],
        );
        assert_eq!(strokes_at_frame_0(&session)[0].samples.len(), 2);
    }

    #[test]
    fn strokes_go_to_the_top_visible_unlocked_layer() {
        let mut project = Project::starter("A");
        let mut top = celframe_core::Layer::new("Top");
        top.locked = true;
        project.scenes[0].layers.push(top);
        let mut session = Session::new(project);
        draw(&mut session, &[(0.0, 0.0), (10.0, 0.0)]);
        let layers = &session.project().scenes[0].layers;
        assert_eq!(layers[0].drawing_at(0).unwrap().strokes.len(), 1);
        assert!(layers[1].drawings.is_empty());
    }

    fn eraser(width: f64, whole_strokes: bool) -> PenOptions {
        PenOptions {
            width,
            tool: PenTool::Erase { whole_strokes },
            ..PenOptions::default()
        }
    }

    /// A session with one horizontal line art stroke from x = -100 to 100 at frame 0.
    fn session_with_a_line() -> Session {
        let mut session = Session::new(Project::starter("A"));
        session
            .pen_down(sample(-100.0, 0.0), 0.1, PenOptions::default())
            .unwrap();
        session.pen_move(&[sample(100.0, 0.0)]).unwrap();
        session.pen_up().unwrap();
        session
    }

    fn shape(shape: crate::pen::ShapeOption) -> PenOptions {
        PenOptions {
            tool: PenTool::Shape { shape },
            ..PenOptions::default()
        }
    }

    const SELECT: PenOptions = PenOptions {
        width: pen::PEN_WIDTH,
        color: pen::PEN_COLOR,
        layer: None,
        art: celframe_core::Art::Line,
        tool: PenTool::Select,
    };

    /// Draws horizontal lines at the given heights, from x = 0 to 100.
    fn session_with_lines(ys: &[f64]) -> Session {
        let mut session = Session::new(Project::starter("A"));
        for &y in ys {
            session
                .pen_down(sample(0.0, y), 0.1, PenOptions::default())
                .unwrap();
            session.pen_move(&[sample(100.0, y)]).unwrap();
            session.pen_up().unwrap();
        }
        session
    }

    fn click(session: &mut Session, x: f64, y: f64) -> DocumentView {
        session.pen_down(sample(x, y), 0.1, SELECT).unwrap();
        session.pen_up().unwrap()
    }

    #[test]
    fn clicking_selects_the_stroke_under_the_pointer_and_empty_space_clears() {
        let mut session = session_with_lines(&[0.0, 50.0]);
        let ids: Vec<_> = strokes_at_frame_0(&session).iter().map(|s| s.id).collect();
        // Half width 2.5 at half pressure, plus 4 device pixels: 8 × 0.1 scene units.
        assert_eq!(click(&mut session, 50.0, 53.2).selection, [ids[1]]);
        assert!(click(&mut session, 50.0, 53.5).selection.is_empty());
        assert!(!session.view().can_redo, "selecting isn't an edit");
    }

    #[test]
    fn shift_click_adds_and_removes() {
        let mut session = session_with_lines(&[0.0, 50.0]);
        let ids: Vec<_> = strokes_at_frame_0(&session).iter().map(|s| s.id).collect();
        click(&mut session, 50.0, 0.0);
        session.set_modifiers(Constraints {
            constrain: true,
            from_centre: false,
        });
        assert_eq!(click(&mut session, 50.0, 50.0).selection, ids);
        assert_eq!(click(&mut session, 50.0, 0.0).selection, [ids[1]]);
    }

    #[test]
    fn a_box_selects_the_strokes_it_touches() {
        let mut session = session_with_lines(&[0.0, 50.0, 100.0]);
        let ids: Vec<_> = strokes_at_frame_0(&session).iter().map(|s| s.id).collect();
        session.pen_down(sample(-20.0, 30.0), 0.1, SELECT).unwrap();
        session.pen_move(&[sample(20.0, 120.0)]).unwrap();
        assert_eq!(
            session.overlay(1.0).len(),
            1,
            "the box is outlined while dragging"
        );
        let view = session.pen_up().unwrap();
        assert_eq!(view.selection, [ids[1], ids[2]]);
        assert_eq!(session.overlay(1.0).len(), 1, "then the selection is");
    }

    #[test]
    fn dragging_a_selection_moves_it_as_one_undo_step() {
        let mut session = session_with_lines(&[0.0]);
        let before = session.project().clone();
        session.pen_down(sample(50.0, 0.0), 0.1, SELECT).unwrap();
        session.pen_move(&[sample(60.0, 10.0)]).unwrap();
        assert_eq!(
            strokes_at_frame_0(&session)[0].samples[0].x,
            10.0,
            "previewed"
        );
        assert!(!session.view().can_redo);
        session.pen_move(&[sample(70.0, 20.0)]).unwrap();
        session.pen_up().unwrap();

        let moved = &strokes_at_frame_0(&session)[0].samples;
        assert_eq!((moved[0].x, moved[0].y), (20.0, 20.0));
        session.undo().unwrap();
        assert_eq!(session.project(), &before);
        session.undo().unwrap();
        assert!(
            strokes_at_frame_0(&session).is_empty(),
            "the move was one step"
        );
    }

    #[test]
    fn undo_during_a_move_drops_the_preview() {
        let mut session = session_with_lines(&[0.0]);
        let drawn = session.project().clone();
        session.pen_down(sample(50.0, 0.0), 0.1, SELECT).unwrap();
        session.pen_move(&[sample(90.0, 0.0)]).unwrap();
        session.clear_selection();
        assert_eq!(session.project(), &drawn);
    }

    #[test]
    fn select_all_delete_and_nudge() {
        let mut session = session_with_lines(&[0.0, 50.0]);
        assert_eq!(
            session
                .select_all(None, celframe_core::Art::Colour)
                .unwrap()
                .selection
                .len(),
            0,
            "only the current part"
        );
        assert_eq!(
            session
                .select_all(None, celframe_core::Art::Line)
                .unwrap()
                .selection
                .len(),
            2
        );
        session.nudge_selection([0.0, -10.0]).unwrap();
        assert_eq!(strokes_at_frame_0(&session)[1].samples[0].y, 40.0);
        let view = session.delete_selection().unwrap();
        assert!(strokes_at_frame_0(&session).is_empty());
        assert!(view.selection.is_empty());
        session.undo().unwrap();
        assert_eq!(
            strokes_at_frame_0(&session).len(),
            2,
            "deleting was one step"
        );
        assert!(session.view().selection.is_empty());
    }

    #[test]
    fn a_dragged_shape_follows_the_pointer_and_is_one_undo_step() {
        let mut session = Session::new(Project::starter("A"));
        session
            .pen_down(
                sample(0.0, 0.0),
                0.1,
                shape(crate::pen::ShapeOption::Rectangle),
            )
            .unwrap();
        session.pen_move(&[sample(10.0, 10.0)]).unwrap();
        session.pen_move(&[sample(40.0, 20.0)]).unwrap();
        session.pen_up().unwrap();

        let strokes = strokes_at_frame_0(&session);
        assert_eq!(strokes.len(), 1);
        let xs: Vec<f64> = strokes[0].samples.iter().map(|s| s.x).collect();
        assert_eq!(strokes[0].samples.len(), 5);
        assert!(
            xs.contains(&40.0) && !xs.contains(&10.0),
            "only the last shape remains"
        );
        assert!(strokes[0].samples.iter().all(|s| s.pressure == 1.0));

        session.undo().unwrap();
        assert!(strokes_at_frame_0(&session).is_empty());
        assert!(!session.view().can_undo, "the shape was a single step");
    }

    #[test]
    fn a_shape_that_never_grows_adds_nothing() {
        let mut session = Session::new(Project::starter("A"));
        session
            .pen_down(
                sample(5.0, 5.0),
                0.1,
                shape(crate::pen::ShapeOption::Ellipse),
            )
            .unwrap();
        session.pen_move(&[sample(5.0, 5.0)]).unwrap();
        session.pen_up().unwrap();
        assert!(strokes_at_frame_0(&session).is_empty());
        assert!(!session.view().can_undo);
    }

    #[test]
    fn modifier_keys_constrain_the_next_movement() {
        let mut session = Session::new(Project::starter("A"));
        session
            .pen_down(
                sample(0.0, 0.0),
                0.1,
                shape(crate::pen::ShapeOption::Rectangle),
            )
            .unwrap();
        session.set_modifiers(Constraints {
            constrain: true,
            from_centre: true,
        });
        session.pen_move(&[sample(10.0, 4.0)]).unwrap();
        session.pen_up().unwrap();
        let samples = &strokes_at_frame_0(&session)[0].samples;
        let max = |f: fn(&StrokeSample) -> f64| samples.iter().map(f).fold(f64::MIN, f64::max);
        let min = |f: fn(&StrokeSample) -> f64| samples.iter().map(f).fold(f64::MAX, f64::min);
        assert_eq!((min(|s| s.x), max(|s| s.x)), (-10.0, 10.0));
        assert_eq!((min(|s| s.y), max(|s| s.y)), (-10.0, 10.0));
    }

    #[test]
    fn a_polygon_with_too_few_sides_is_refused() {
        let mut session = Session::new(Project::starter("A"));
        let options = shape(crate::pen::ShapeOption::Polygon { sides: 2 });
        assert!(session.pen_down(sample(0.0, 0.0), 0.1, options).is_err());
    }

    #[test]
    fn the_first_stroke_on_an_empty_frame_is_one_undo_step() {
        let mut project = Project::starter("A");
        let layer = &mut project.scenes[0].layers[0];
        layer.exposures.clear();
        layer.drawings.clear();
        let mut session = Session::new(project);
        session
            .pen_down(sample(0.0, 0.0), 0.1, PenOptions::default())
            .unwrap();
        session.pen_move(&[sample(20.0, 0.0)]).unwrap();
        session.pen_up().unwrap();
        assert_eq!(strokes_at_frame_0(&session).len(), 1);
        session.undo().unwrap();
        assert!(!session.view().can_undo);
        assert!(session.project().scenes[0].layers[0].drawings.is_empty());
    }

    #[test]
    fn a_drag_of_the_eraser_cuts_strokes_and_undoes_as_one_step() {
        let mut session = session_with_a_line();
        let drawn = session.project().clone();

        session
            .pen_down(sample(-50.0, 20.0), 0.1, eraser(10.0, false))
            .unwrap();
        session.pen_move(&[sample(-50.0, -20.0)]).unwrap();
        session.pen_move(&[sample(50.0, -20.0)]).unwrap();
        session.pen_move(&[sample(50.0, 20.0)]).unwrap();
        session.pen_up().unwrap();

        let pieces = strokes_at_frame_0(&session);
        assert_eq!(pieces.len(), 3, "two cuts leave three pieces");
        assert!(pieces.iter().all(|s| s.base_width == pen::PEN_WIDTH));

        session.undo().unwrap();
        assert_eq!(session.project(), &drawn, "one undo restores the stroke");
    }

    #[test]
    fn whole_strokes_mode_removes_a_touched_stroke() {
        let mut session = session_with_a_line();
        session
            .pen_down(sample(0.0, 0.0), 0.1, eraser(4.0, true))
            .unwrap();
        session.pen_up().unwrap();
        assert!(strokes_at_frame_0(&session).is_empty());
    }

    #[test]
    fn the_eraser_only_erases_the_current_part() {
        let mut session = session_with_a_line();
        let colour = PenOptions {
            art: celframe_core::Art::Colour,
            ..eraser(40.0, true)
        };
        session.pen_down(sample(0.0, 0.0), 0.1, colour).unwrap();
        session.pen_up().unwrap();
        assert_eq!(strokes_at_frame_0(&session).len(), 1);
    }

    #[test]
    fn an_eraser_that_touches_nothing_adds_no_undo_step() {
        let mut session = session_with_a_line();
        session
            .pen_down(sample(0.0, 500.0), 0.1, eraser(10.0, false))
            .unwrap();
        session.pen_move(&[sample(10.0, 500.0)]).unwrap();
        session.pen_up().unwrap();
        session.undo().unwrap();
        assert!(
            strokes_at_frame_0(&session).is_empty(),
            "undo removed the line"
        );
    }

    #[test]
    fn the_chosen_brush_size_colour_and_layer_are_used() {
        let mut project = Project::starter("A");
        project.scenes[0]
            .layers
            .push(celframe_core::Layer::new("Top"));
        let bottom = project.scenes[0].layers[0].id;
        let mut session = Session::new(project);
        let options = PenOptions {
            width: 3.5,
            color: [200, 10, 20, 255],
            layer: Some(bottom),
            art: celframe_core::Art::Colour,
            tool: PenTool::Draw,
        };
        session.pen_down(sample(0.0, 0.0), 0.1, options).unwrap();
        session.pen_up().unwrap();
        let stroke = &strokes_at_frame_0(&session)[0];
        assert_eq!(stroke.base_width, 3.5);
        assert_eq!(stroke.color, [200, 10, 20, 255]);
        assert_eq!(stroke.art, celframe_core::Art::Colour);
        assert!(session.project().scenes[0].layers[1].drawings.is_empty());
    }

    #[test]
    fn a_hidden_chosen_layer_or_a_bad_size_is_refused() {
        let mut project = Project::starter("A");
        project.scenes[0].layers[0].visible = false;
        let layer = project.scenes[0].layers[0].id;
        let mut session = Session::new(project);
        let hidden = PenOptions {
            layer: Some(layer),
            ..PenOptions::default()
        };
        assert!(session.pen_down(sample(0.0, 0.0), 0.1, hidden).is_err());
        let huge = PenOptions {
            width: 1e9,
            ..PenOptions::default()
        };
        assert!(session.pen_down(sample(0.0, 0.0), 0.1, huge).is_err());
        assert!(!session.view().can_undo);
    }

    #[test]
    fn add_layer_puts_a_numbered_layer_on_top_and_undoes() {
        let mut session = Session::new(Project::new("A"));
        let view = session.add_layer().unwrap();
        let layers = &view.project.scenes[0].layers;
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[1].name, "Layer 2");
        assert_eq!(layers[1].id.as_uuid().get_version_num(), 7);
        assert_eq!(session.undo().unwrap().project.scenes[0].layers.len(), 1);
    }

    #[test]
    fn a_layer_with_no_drawing_gets_one_first() {
        let mut session = Session::new(Project::new("A"));
        draw(&mut session, &[(0.0, 0.0), (10.0, 0.0)]);
        assert_eq!(strokes_at_frame_0(&session).len(), 1);
    }

    #[test]
    fn with_no_layer_to_draw_on_the_pen_explains_why() {
        let mut project = Project::starter("A");
        project.scenes[0].layers[0].visible = false;
        let mut session = Session::new(project);
        let error = session
            .pen_down(sample(0.0, 0.0), 0.1, PenOptions::default())
            .unwrap_err();
        assert!(error.contains("unlock a layer"), "{error}");
        assert!(!session.view().dirty);
    }

    #[test]
    fn invalid_positions_and_stray_moves_are_ignored_safely() {
        let mut session = Session::new(Project::starter("A"));
        assert!(
            session
                .pen_down(sample(f64::NAN, 0.0), 0.1, PenOptions::default())
                .is_err()
        );
        session.pen_move(&[sample(1.0, 1.0)]).unwrap();
        session.pen_up().unwrap();
        assert!(strokes_at_frame_0(&session).is_empty());
    }

    #[test]
    fn undo_mid_stroke_ends_the_stroke() {
        let mut session = Session::new(Project::starter("A"));
        session
            .pen_down(sample(0.0, 0.0), 0.1, PenOptions::default())
            .unwrap();
        session
            .pen_move(&[sample(5.0, 0.0), sample(10.0, 0.0), sample(15.0, 0.0)])
            .unwrap();
        session.undo().unwrap();
        session.pen_move(&[sample(20.0, 0.0)]).unwrap();
        session.pen_up().unwrap();
        assert!(strokes_at_frame_0(&session).is_empty());
    }

    #[test]
    fn new_documents_are_ready_to_draw_on() {
        let mut session = Session::new(Project::new("Old"));
        session.new_document();
        draw(&mut session, &[(0.0, 0.0), (10.0, 0.0)]);
        session.undo().unwrap();
        // One undo removed the stroke and nothing is left to undo.
        assert!(!session.view().can_undo);
    }

    #[test]
    fn failed_save_keeps_the_project_dirty_and_unsaved() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("no-such-folder").join("a.celf");
        let mut session = Session::new(Project::new("A"));
        session.execute(rename("B")).unwrap();

        let error = session.save_to(&missing).unwrap_err();
        assert!(error.starts_with("Couldn't save a.celf: "), "{error}");
        let view = session.view();
        assert!(view.dirty && view.path.is_none());
    }
}
