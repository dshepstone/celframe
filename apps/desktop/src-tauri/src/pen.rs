//! A stroke being drawn with the pen. See `docs/specs/pen-input.md`.

use celframe_core::{Art, DrawingId, LayerId, Project, SceneId, Stroke, StrokeId, StrokeSample};
use celframe_geom::StrokeSmoother;
use serde::Deserialize;

/// The pen's defaults, used when the UI sends no options.
pub const PEN_WIDTH: f64 = 10.0;
pub const PEN_COLOR: [u8; 4] = [0, 0, 0, 255];

/// Widths the pen accepts, in scene units at full pressure.
pub const PEN_WIDTH_RANGE: std::ops::RangeInclusive<f64> = 0.5..=200.0;

/// What the UI chose for the next stroke.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PenOptions {
    pub width: f64,
    pub color: [u8; 4],
    /// The layer to draw on. None means the topmost visible, unlocked layer.
    pub layer: Option<LayerId>,
    /// The part of the drawing to draw on. Line art if the UI sends none.
    #[serde(default)]
    pub art: Art,
    /// Whether the pen draws or erases. Drawing if the UI sends none.
    #[serde(default)]
    pub tool: PenTool,
}

/// What a pen drag does. When erasing, `width` is the eraser's diameter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PenTool {
    #[default]
    Draw,
    Erase {
        /// Remove every stroke the eraser touches, instead of cutting it.
        whole_strokes: bool,
    },
    /// Drag out a shape with the pen's size and colour.
    Shape { shape: ShapeOption },
    /// Select and move strokes.
    Select {
        /// Dragging from empty space draws a lasso instead of a box.
        #[serde(default)]
        lasso: bool,
        /// Scaling with handles also scales stroke widths.
        #[serde(default)]
        scale_widths: bool,
    },
}

/// A shape as the UI names it. See `docs/specs/shape-tools.md`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ShapeOption {
    Line,
    Rectangle,
    Ellipse,
    Polygon { sides: u32 },
}

impl From<ShapeOption> for celframe_geom::Shape {
    fn from(shape: ShapeOption) -> Self {
        match shape {
            ShapeOption::Line => Self::Line,
            ShapeOption::Rectangle => Self::Rectangle,
            ShapeOption::Ellipse => Self::Ellipse,
            ShapeOption::Polygon { sides } => Self::Polygon { sides },
        }
    }
}

impl Default for PenOptions {
    fn default() -> Self {
        Self {
            width: PEN_WIDTH,
            color: PEN_COLOR,
            layer: None,
            art: Art::Line,
            tool: PenTool::Draw,
        }
    }
}

/// Where a new stroke goes, in the first scene, and the drawing it shows at `frame` if any.
/// A chosen layer must be visible and unlocked. Without one, the topmost such layer is used.
pub fn find_target(
    project: &Project,
    frame: u32,
    chosen: Option<LayerId>,
) -> Result<(SceneId, LayerId, Option<DrawingId>), &'static str> {
    let scene = project
        .scenes
        .first()
        .ok_or("The project has no scene to draw in.")?;
    let layer = match chosen {
        Some(id) => {
            let layer = scene
                .layers
                .iter()
                .find(|layer| layer.id == id)
                .ok_or("That layer no longer exists.")?;
            if !layer.visible || layer.locked {
                return Err("Show and unlock the current layer to draw on it.");
            }
            layer
        }
        None => scene
            .layers
            .iter()
            .rev()
            .find(|layer| layer.visible && !layer.locked)
            .ok_or("Show or unlock a layer to draw on it.")?,
    };
    Ok((scene.id, layer.id, layer.drawing_at(frame).map(|d| d.id)))
}

/// Cleans a raw sample the way the smoother does. Returns `None` for non-finite values.
pub fn clean(sample: StrokeSample) -> Option<StrokeSample> {
    (sample.x.is_finite() && sample.y.is_finite() && sample.pressure.is_finite()).then(|| {
        StrokeSample {
            pressure: sample.pressure.clamp(0.0, 1.0),
            ..sample
        }
    })
}

pub struct PenStroke {
    pub scene: SceneId,
    pub layer: LayerId,
    pub drawing: DrawingId,
    pub stroke: StrokeId,
    smoother: StrokeSmoother,
    /// The stroke starts with its first sample, which the smoother also emits later.
    first_pending: bool,
}

impl PenStroke {
    /// Starts smoothing from `first`, which must already be clean. Returns the stroke to add.
    pub fn start(
        target: (SceneId, LayerId, DrawingId),
        first: StrokeSample,
        min_distance: f64,
        options: PenOptions,
    ) -> (Self, Stroke) {
        let stroke = Stroke {
            id: StrokeId::new(),
            base_width: options.width,
            color: options.color,
            art: options.art,
            samples: vec![first],
        };
        let mut smoother = StrokeSmoother::new(min_distance);
        let emitted = smoother.push(first);
        debug_assert!(emitted.is_empty(), "one sample is never final");
        let pen = Self {
            scene: target.0,
            layer: target.1,
            drawing: target.2,
            stroke: stroke.id,
            smoother,
            first_pending: true,
        };
        (pen, stroke)
    }

    /// Adds raw samples and returns the smoothed ones that are now final and new to the stroke.
    pub fn push(&mut self, samples: &[StrokeSample]) -> Vec<StrokeSample> {
        let mut output = Vec::new();
        for sample in samples {
            output.extend(self.smoother.push(*sample));
        }
        self.drop_first(output)
    }

    /// Ends the stroke and returns its remaining samples.
    pub fn finish(mut self) -> Vec<StrokeSample> {
        let output = std::mem::replace(&mut self.smoother, StrokeSmoother::new(0.0)).finish();
        self.drop_first(output)
    }

    fn drop_first(&mut self, mut output: Vec<StrokeSample>) -> Vec<StrokeSample> {
        if self.first_pending && !output.is_empty() {
            output.remove(0);
            self.first_pending = false;
        }
        output
    }
}
