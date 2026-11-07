//! Selecting and moving strokes. See `docs/specs/selection.md`.

use celframe_core::{
    Art, Command, Drawing, DrawingId, LayerId, Project, SceneId, Stroke, StrokeId, StrokeSample,
};
use celframe_geom::{distance_to_stroke, stroke_bounds, stroke_touches_box, union};

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
