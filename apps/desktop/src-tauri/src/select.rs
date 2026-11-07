//! Selecting and moving strokes. See `docs/specs/selection.md`.

use celframe_core::{
    Art, Command, Drawing, DrawingId, LayerId, Project, SceneId, Stroke, StrokeId, StrokeSample,
};
use celframe_geom::{
    Affine, centreline_bounds, distance_to_stroke, stroke_bounds, stroke_touches_box,
    stroke_touches_lasso, union,
};
use serde::Deserialize;

/// The selection outline's colour and widths in device pixels.
const OUTLINE: [u8; 4] = [40, 120, 240, 255];
const HANDLE_FILL: [u8; 4] = [255, 255, 255, 255];
const SELECTION_WIDTH: f64 = 1.5;
const MARQUEE_WIDTH: f64 = 1.0;
/// Handle size, how far beyond it a handle can still be grabbed, and how far the rotate
/// handle sits above the box, all in device pixels. See `docs/specs/transform-and-arrange.md`.
const HANDLE_SIZE: f64 = 8.0;
const HANDLE_REACH: f64 = 5.0;
const ROTATE_OFFSET: f64 = 24.0;
const ROTATE_SNAP: f64 = 15.0;

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
        Ok(Some(self.transformed(project, transform, width_scale)))
    }

    /// Replaces each selected stroke with itself transformed, widths scaled by `width_scale`.
    pub fn transformed(&self, project: &Project, transform: Affine, width_scale: f64) -> Command {
        self.replaced(project, |stroke| {
            transform_stroke(stroke, transform, width_scale)
        })
    }

    /// The selected strokes as they are now, to transform from.
    pub fn stroke_originals(&self, project: &Project) -> Vec<Stroke> {
        self.existing(project).into_iter().cloned().collect()
    }

    /// Writes `strokes` over the selected strokes with the same IDs, without recording an
    /// edit. Used only to preview a drag; the drag's end restores the originals first.
    pub fn preview(&self, project: &mut Project, strokes: &[Stroke]) {
        let Some(drawing) = project
            .scenes
            .iter_mut()
            .find(|s| s.id == self.scene)
            .and_then(|s| s.layers.iter_mut().find(|l| l.id == self.layer))
            .and_then(|l| l.drawings.iter_mut().find(|d| d.id == self.drawing))
        else {
            return;
        };
        for stroke in &mut drawing.strokes {
            if let Some(new) = strokes.iter().find(|s| s.id == stroke.id) {
                stroke.clone_from(new);
            }
        }
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

fn transform_stroke(stroke: &Stroke, transform: Affine, width_scale: f64) -> Stroke {
    Stroke {
        base_width: stroke.base_width * width_scale,
        ..transform.apply_to_stroke(stroke)
    }
}

/// A handle on the selection's box: a side or corner given by which way it points, or the
/// rotate handle. `x` and `y` are -1, 0 or 1; a corner has neither 0, a side has one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    Scale { x: i8, y: i8 },
    Rotate,
}

const SCALE_HANDLES: [(i8, i8); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

fn centre(area: [f64; 4]) -> [f64; 2] {
    [(area[0] + area[2]) / 2.0, (area[1] + area[3]) / 2.0]
}

/// Where a scale handle sits on the box: -1 is the low edge, 1 the high edge, 0 the middle.
fn on_box(area: [f64; 4], x: i8, y: i8) -> [f64; 2] {
    let pick = |side: i8, low: f64, high: f64| match side {
        -1 => low,
        1 => high,
        _ => (low + high) / 2.0,
    };
    [pick(x, area[0], area[2]), pick(y, area[1], area[3])]
}

/// Every handle and where it is, for a box `area`. `pixel` is one device pixel in scene units.
pub fn handles(area: [f64; 4], pixel: f64) -> Vec<(Handle, [f64; 2])> {
    let top = [(area[0] + area[2]) / 2.0, area[3] + ROTATE_OFFSET * pixel];
    SCALE_HANDLES
        .iter()
        .map(|&(x, y)| (Handle::Scale { x, y }, on_box(area, x, y)))
        .chain([(Handle::Rotate, top)])
        .collect()
}

/// The handle within reach of `point`, nearest first.
pub fn handle_at(area: [f64; 4], point: [f64; 2], pixel: f64) -> Option<Handle> {
    let reach = (HANDLE_SIZE / 2.0 + HANDLE_REACH) * pixel;
    handles(area, pixel)
        .into_iter()
        .map(|(handle, at)| (handle, (at[0] - point[0]).hypot(at[1] - point[1])))
        .filter(|(_, distance)| *distance <= reach)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(handle, _)| handle)
}

/// The transform for dragging `handle` on the box `area` from `start` to `point`, and the
/// matching stroke-width scale. `constrain` is Shift and `from_centre` is Alt.
pub fn handle_transform(
    handle: Handle,
    area: [f64; 4],
    start: [f64; 2],
    point: [f64; 2],
    constrain: bool,
    from_centre: bool,
) -> (Affine, [f64; 2]) {
    let middle = centre(area);
    match handle {
        Handle::Rotate => {
            let angle = |p: [f64; 2]| (p[1] - middle[1]).atan2(p[0] - middle[0]);
            let mut degrees = (angle(point) - angle(start)).to_degrees();
            if constrain {
                degrees = (degrees / ROTATE_SNAP).round() * ROTATE_SNAP;
            }
            (Affine::rotate_about(middle, degrees), [1.0, 1.0])
        }
        Handle::Scale { x, y } => {
            let at = on_box(area, x, y);
            let anchor = if from_centre {
                middle
            } else {
                on_box(area, -x, -y)
            };
            let moved = [at[0] + point[0] - start[0], at[1] + point[1] - start[1]];
            let factor = |side: i8, axis: usize| {
                let span = at[axis] - anchor[axis];
                if side == 0 || span == 0.0 {
                    1.0
                } else {
                    (moved[axis] - anchor[axis]) / span
                }
            };
            let (mut sx, mut sy) = (factor(x, 0), factor(y, 1));
            if constrain && x != 0 && y != 0 {
                let s = if sx.abs() > sy.abs() { sx } else { sy };
                (sx, sy) = (s, s);
            }
            (Affine::scale_about(anchor, sx, sy), [sx, sy])
        }
    }
}

/// The handle transform applied to the drag's original strokes.
pub fn transform_originals(
    originals: &[Stroke],
    transform: Affine,
    width_scale: f64,
) -> Vec<Stroke> {
    originals
        .iter()
        .map(|s| transform_stroke(s, transform, width_scale))
        .collect()
}

/// How stroke widths scale for handle scale factors, when widths scale at all.
pub fn width_scale(factors: [f64; 2], scale_widths: bool) -> f64 {
    if scale_widths {
        (factors[0].abs() + factors[1].abs()) / 2.0
    } else {
        1.0
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
    /// Drawing a lasso. With `add`, the strokes in it join the selection.
    Lasso { points: Vec<[f64; 2]>, add: bool },
    /// Dragging a handle on the selection's box, from the strokes as they were.
    Transform {
        handle: Handle,
        area: [f64; 4],
        start: [f64; 2],
        last: [f64; 2],
        originals: Vec<Stroke>,
        scale_widths: bool,
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

/// The strokes of `art` with any part inside the lasso.
pub fn in_lasso(drawing: &Drawing, art: Art, lasso: &[[f64; 2]]) -> Vec<StrokeId> {
    drawing
        .strokes
        .iter()
        .filter(|s| s.art == art && stroke_touches_lasso(s, lasso))
        .map(|s| s.id)
        .collect()
}

pub fn contains(area: [f64; 4], point: [f64; 2]) -> bool {
    point[0] >= area[0] && point[0] <= area[2] && point[1] >= area[1] && point[1] <= area[3]
}

/// What the canvas draws over the scene for the Select tool.
#[derive(Debug, Default)]
pub struct Overlay<'a> {
    /// The selection's painted box.
    pub selection: Option<[f64; 4]>,
    /// Draw handles on the selection's box.
    pub handles: bool,
    /// A box being dragged out.
    pub marquee: Option<[f64; 4]>,
    /// A lasso being drawn.
    pub lasso: Option<&'a [[f64; 2]]>,
}

fn line(points: impl IntoIterator<Item = [f64; 2]>, width: f64, color: [u8; 4]) -> Stroke {
    Stroke {
        id: StrokeId::from_u128(0),
        base_width: width,
        color,
        art: Art::Line,
        samples: points
            .into_iter()
            .map(|[x, y]| StrokeSample {
                x,
                y,
                pressure: 1.0,
            })
            .collect(),
    }
}

fn outline(area: [f64; 4], width: f64) -> Stroke {
    line(
        [
            [area[0], area[1]],
            [area[2], area[1]],
            [area[2], area[3]],
            [area[0], area[3]],
            [area[0], area[1]],
        ],
        width,
        OUTLINE,
    )
}

/// A filled square `size` across: a square path half that size, stroked half that wide.
fn square(at: [f64; 2], size: f64, color: [u8; 4]) -> Stroke {
    let h = size / 4.0;
    outline_with(
        [at[0] - h, at[1] - h, at[0] + h, at[1] + h],
        size / 2.0,
        color,
    )
}

fn outline_with(area: [f64; 4], width: f64, color: [u8; 4]) -> Stroke {
    Stroke {
        color,
        ..outline(area, width)
    }
}

/// A filled circle `size` across: a single round point.
fn dot(at: [f64; 2], size: f64, color: [u8; 4]) -> Stroke {
    line([at], size, color)
}

impl Overlay<'_> {
    /// Strokes to draw over the scene, bottom first. `pixel` is one device pixel in scene units.
    pub fn strokes(&self, pixel: f64) -> Vec<Stroke> {
        let mut strokes = Vec::new();
        if let Some(area) = self.selection {
            strokes.push(outline(area, SELECTION_WIDTH * pixel));
            if self.handles {
                let top = [(area[0] + area[2]) / 2.0, area[3]];
                for (handle, at) in handles(area, pixel) {
                    let size = HANDLE_SIZE * pixel;
                    let edge = 1.5 * pixel;
                    if handle == Handle::Rotate {
                        strokes.push(line([top, at], pixel, OUTLINE));
                        strokes.push(dot(at, size + 2.0 * edge, OUTLINE));
                        strokes.push(dot(at, size, HANDLE_FILL));
                    } else {
                        strokes.push(square(at, size + 2.0 * edge, OUTLINE));
                        strokes.push(square(at, size, HANDLE_FILL));
                    }
                }
            }
        }
        if let Some(area) = self.marquee {
            strokes.push(outline(area, MARQUEE_WIDTH * pixel));
        }
        if let Some(points) = self.lasso.filter(|p| p.len() >= 2) {
            let closed = points.iter().copied().chain(points.first().copied());
            strokes.push(line(closed, MARQUEE_WIDTH * pixel, OUTLINE));
        }
        strokes
    }
}
