//! Erasing with the pen. See `docs/specs/eraser.md`.

use celframe_core::{
    Art, Command, DrawingId, LayerId, Project, SceneId, Stroke, StrokeId, StrokeSample,
};
use celframe_geom::{Erased, erase};

/// An eraser drag in progress.
pub struct EraserDrag {
    pub scene: SceneId,
    pub layer: LayerId,
    /// The drawing shown at the current frame. None when there is nothing to erase.
    pub drawing: Option<DrawingId>,
    pub art: Art,
    pub radius: f64,
    pub whole_strokes: bool,
    /// Where the eraser was last, so the next move erases along the line from there.
    pub last: [f64; 2],
}

impl EraserDrag {
    /// The edits that erase along the eraser's path from where it was to `samples`, then moves
    /// it there. Empty when nothing was touched.
    pub fn erase_to(&mut self, project: &Project, samples: &[StrokeSample]) -> Vec<Command> {
        let mut path = vec![self.last];
        path.extend(samples.iter().map(|s| [s.x, s.y]));
        self.last = *path.last().expect("the path starts with the last point");
        self.commands(project, &path)
    }

    fn commands(&self, project: &Project, path: &[[f64; 2]]) -> Vec<Command> {
        let Some(drawing) = self.drawing else {
            return Vec::new();
        };
        let Some(strokes) = project
            .scenes
            .iter()
            .find(|s| s.id == self.scene)
            .and_then(|s| s.layers.iter().find(|l| l.id == self.layer))
            .and_then(|l| l.drawings.iter().find(|d| d.id == drawing))
            .map(|d| &d.strokes)
        else {
            return Vec::new();
        };
        let (scene, layer) = (self.scene, self.layer);
        let mut commands = Vec::new();
        // From the top down, so each edit leaves the indices below it unchanged.
        for (index, stroke) in strokes.iter().enumerate().rev() {
            if stroke.art != self.art {
                continue;
            }
            let Erased::Pieces(pieces) = erase(&stroke.samples, path, self.radius) else {
                continue;
            };
            commands.push(Command::RemoveStroke {
                scene,
                layer,
                drawing,
                stroke: stroke.id,
            });
            if self.whole_strokes {
                continue;
            }
            for (offset, samples) in pieces.into_iter().enumerate() {
                commands.push(Command::AddStroke {
                    scene,
                    layer,
                    drawing,
                    stroke: Stroke {
                        id: StrokeId::new(),
                        samples,
                        ..stroke.clone()
                    },
                    index: index + offset,
                });
            }
        }
        commands
    }
}
