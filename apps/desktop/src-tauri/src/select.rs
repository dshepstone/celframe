//! Selecting and moving strokes. See `docs/specs/selection.md`.

use celframe_core::{
    Art, Command, Drawing, DrawingId, LayerId, Project, SceneId, Stroke, StrokeId, StrokeSample,
};
use celframe_geom::{
    Affine, centreline_bounds, distance_to_stroke, stroke_bounds, stroke_touches_box, union,
};
use serde::Deserialize;

/// The selection outline's colour and widths in device pixels.
const OUTLINE: [u8; 4] = [40, 120, 240, 255];
const SELECTION_WIDTH: f64 = 1.5;
const MARQUEE_WIDTH: f64 = 1.0;

/// Selected strokes, all in one part of one drawing.
#[derive(Clone, Debug, PartialEq)]
pub struct Selection {
    pub scene: SceneId,
    pub layer: LayerId,
    pub drawing: DrawingId,
    pub art: Art,
    pub strokes: Vec<StrokeId>,
}

impl Selection {
    /// The selected strokes that still exist, bottom first.
    pub fn existing<'a>(&self, project: &'a Project) -> Vec<&'a Stroke> {
        drawing(project, self.scene, self.layer, self.drawing).map_or_else(Vec::new, |d| {
            d.strokes
                .iter()
                .filter(|s| self.strokes.contains(&s.id))
                .collect()
        })
    }

    pub fn bounds(&self, project: &Project) -> Option<[f64; 4]> {
        self.existing(project)
            .into_iter()
            .filter_map(stroke_bounds)
            .reduce(union)
    }

    /// Edits that move the selected strokes from `originals` by `delta`.
    pub fn moved(&self, originals: &[(StrokeId, Vec<StrokeSample>)], delta: [f64; 2]) -> Command {
        let (scene, layer, drawing) = (self.scene, self.layer, self.drawing);
        let commands = originals
            .iter()
            .flat_map(|(stroke, samples)| {
                let samples = samples
                    .iter()
                    .map(|s| StrokeSample {
                        x: s.x + delta[0],
                        y: s.y + delta[1],
                        ..*s
                    })
                    .collect();
                [
                    Command::TruncateStrokeSamples {
                        scene,
                        layer,
                        drawing,
                        stroke: *stroke,
                        len: 0,
                    },
                    Command::AppendStrokeSamples {
                        scene,
                        layer,
                        drawing,
                        stroke: *stroke,
                        samples,
                    },
                ]
            })
            .collect();
        Command::Batch { commands }
    }

    /// The selected strokes' samples, to move from.
    pub fn originals(&self, project: &Project) -> Vec<(StrokeId, Vec<StrokeSample>)> {
        self.existing(project)
            .into_iter()
            .map(|s| (s.id, s.samples.clone()))
            .collect()
    }
}

/// An edit to the whole selection. See `docs/specs/transform-and-arrange.md`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SelectionEdit {
    /// Moves the centreline box's centre to `x`, `y` and scales it to `width` by `height`.
    SetBounds {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        scale_widths: bool,
    },
    /// Turns counter-clockwise about the box's centre.
    Rotate {
        degrees: f64,
    },
    /// Mirrors about the box's centre.
    Flip {
        horizontal: bool,
    },
    Arrange {
        to: Arrange,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Arrange {
    Front,
    Forward,
    Backward,
    Back,
}

impl Selection {
    /// The box around the selected strokes' centrelines.
    pub fn centreline_bounds(&self, project: &Project) -> Option<[f64; 4]> {
        centreline_bounds(self.existing(project))
    }

    /// The command for an edit, or None when it changes nothing.
    pub fn edit(&self, project: &Project, edit: SelectionEdit) -> Result<Option<Command>, String> {
        if let SelectionEdit::Arrange { to } = edit {
            return Ok(self.arrange(project, to));
        }
        let Some([x0, y0, x1, y1]) = self.centreline_bounds(project) else {
            return Ok(None);
        };
        let centre = [(x0 + x1) / 2.0, (y0 + y1) / 2.0];
        let (transform, width_scale) = match edit {
            SelectionEdit::SetBounds {
                x,
                y,
                width,
                height,
                scale_widths,
            } => {
                let valid = |v: f64| v.is_finite() && v > 0.0;
                if !x.is_finite() || !y.is_finite() || !valid(width) || !valid(height) {
                    return Err("Those sizes aren't valid.".to_owned());
                }
                // An axis with no extent can't be scaled.
                let factor = |target: f64, current: f64| {
                    if current > 0.0 { target / current } else { 1.0 }
                };
                let (sx, sy) = (factor(width, x1 - x0), factor(height, y1 - y0));
                let mut transform = Affine::scale_about(centre, sx, sy);
                transform.e += x - centre[0];
                transform.f += y - centre[1];
                let widths = if scale_widths { (sx + sy) / 2.0 } else { 1.0 };
                (transform, widths)
            }
            SelectionEdit::Rotate { degrees } => {
                if !degrees.is_finite() {
                    return Err("That angle isn't valid.".to_owned());
                }
                (Affine::rotate_about(centre, degrees), 1.0)
            }
            SelectionEdit::Flip { horizontal } => {
                let (sx, sy) = if horizontal { (-1.0, 1.0) } else { (1.0, -1.0) };
                (Affine::scale_about(centre, sx, sy), 1.0)
            }
            SelectionEdit::Arrange { .. } => unreachable!("handled above"),
        };
        if transform == Affine::IDENTITY && width_scale == 1.0 {
            return Ok(None);
        }
        Ok(Some(self.replaced(project, |stroke| Stroke {
            base_width: stroke.base_width * width_scale,
            ..transform.apply_to_stroke(stroke)
        })))
    }

    /// Replaces each selected stroke with `change(stroke)`, keeping its ID and position.
    fn replaced(&self, project: &Project, change: impl Fn(&Stroke) -> Stroke) -> Command {
        let (scene, layer, drawing) = (self.scene, self.layer, self.drawing);
        let strokes = self::drawing(project, scene, layer, drawing).map_or(&[][..], |d| &d.strokes);
        let commands = strokes
            .iter()
            .enumerate()
            .filter(|(_, s)| self.strokes.contains(&s.id))
            .flat_map(|(index, stroke)| {
                [
                    Command::RemoveStroke {
                        scene,
                        layer,
                        drawing,
                        stroke: stroke.id,
                    },
                    Command::AddStroke {
                        scene,
                        layer,
                        drawing,
                        stroke: change(stroke),
                        index,
                    },
                ]
            })
            .collect();
        Command::Batch { commands }
    }

    fn arrange(&self, project: &Project, to: Arrange) -> Option<Command> {
        let (scene, layer, drawing) = (self.scene, self.layer, self.drawing);
        let strokes = &self::drawing(project, scene, layer, drawing)?.strokes;
        let picked = |s: &&Stroke| self.strokes.contains(&s.id);
        let mut order: Vec<&Stroke> = strokes.iter().collect();
        match to {
            Arrange::Front => order.sort_by_key(|s| picked(s)),
            Arrange::Back => order.sort_by_key(|s| !picked(s)),
            Arrange::Forward => {
                for i in (0..order.len().saturating_sub(1)).rev() {
                    if picked(&order[i]) && !picked(&order[i + 1]) {
                        order.swap(i, i + 1);
                    }
                }
            }
            Arrange::Backward => {
                for i in 1..order.len() {
                    if picked(&order[i]) && !picked(&order[i - 1]) {
                        order.swap(i, i - 1);
                    }
                }
            }
        }
        if order.iter().zip(strokes).all(|(a, b)| a.id == b.id) {
            return None;
        }
        // Take the selected strokes out, then put each back at its new index, lowest first.
        let removals = strokes
            .iter()
            .filter(picked)
            .map(|s| Command::RemoveStroke {
                scene,
                layer,
                drawing,
                stroke: s.id,
            });
        let insertions = order
            .iter()
            .enumerate()
            .filter(|(_, s)| picked(s))
            .map(|(index, s)| Command::AddStroke {
                scene,
                layer,
                drawing,
                stroke: (*s).clone(),
                index,
            });
        Some(Command::Batch {
            commands: removals.chain(insertions).collect(),
        })
    }
}

/// A drag with the Select tool.
pub enum SelectDrag {
    /// Moving the selection. A click that doesn't move makes `click` the selection, if set.
    Move {
        start: [f64; 2],
        last: [f64; 2],
        originals: Vec<(StrokeId, Vec<StrokeSample>)>,
        click: Option<Vec<StrokeId>>,
    },
    /// Dragging out a box. With `add`, the strokes in it join the selection.
    Marquee {
        start: [f64; 2],
        last: [f64; 2],
        add: bool,
    },
}

pub fn drawing(
    project: &Project,
    scene: SceneId,
    layer: LayerId,
    drawing: DrawingId,
) -> Option<&Drawing> {
    project
        .scenes
        .iter()
        .find(|s| s.id == scene)?
        .layers
        .iter()
        .find(|l| l.id == layer)?
        .drawings
        .iter()
        .find(|d| d.id == drawing)
}

/// The topmost stroke of `art` within `tolerance` of `point`.
pub fn hit(drawing: &Drawing, art: Art, point: [f64; 2], tolerance: f64) -> Option<StrokeId> {
    drawing
        .strokes
        .iter()
        .rev()
        .filter(|s| s.art == art)
        .find(|s| distance_to_stroke(s, point) <= tolerance)
        .map(|s| s.id)
}

/// The strokes of `art` with any part inside the box between two corners.
pub fn in_box(drawing: &Drawing, art: Art, a: [f64; 2], b: [f64; 2]) -> Vec<StrokeId> {
    let area = [
        a[0].min(b[0]),
        a[1].min(b[1]),
        a[0].max(b[0]),
        a[1].max(b[1]),
    ];
    drawing
        .strokes
        .iter()
        .filter(|s| s.art == art && stroke_touches_box(s, area))
        .map(|s| s.id)
        .collect()
}

pub fn contains(area: [f64; 4], point: [f64; 2]) -> bool {
    point[0] >= area[0] && point[0] <= area[2] && point[1] >= area[1] && point[1] <= area[3]
}

/// Outlines to draw over the scene: the selection's box and any box being dragged out.
/// `pixel` is one device pixel in scene units.
pub fn overlay(selection: Option<[f64; 4]>, marquee: Option<[f64; 4]>, pixel: f64) -> Vec<Stroke> {
    let outline = |area: [f64; 4], width: f64| Stroke {
        id: StrokeId::from_u128(0),
        base_width: width * pixel,
        color: OUTLINE,
        art: Art::Line,
        samples: [
            [area[0], area[1]],
            [area[2], area[1]],
            [area[2], area[3]],
            [area[0], area[3]],
            [area[0], area[1]],
        ]
        .into_iter()
        .map(|[x, y]| StrokeSample {
            x,
            y,
            pressure: 1.0,
        })
        .collect(),
    };
    selection
        .map(|a| outline(a, SELECTION_WIDTH))
        .into_iter()
        .chain(marquee.map(|a| outline(a, MARQUEE_WIDTH)))
        .collect()
}
