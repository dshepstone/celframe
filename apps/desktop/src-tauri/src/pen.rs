//! A stroke being drawn with the pen. See `docs/specs/pen-input.md`.

use celframe_core::{DrawingId, LayerId, Project, SceneId, Stroke, StrokeId, StrokeSample};
use celframe_geom::StrokeSmoother;

/// Phase 1's one pen.
pub const PEN_WIDTH: f64 = 10.0;
pub const PEN_COLOR: [u8; 4] = [0, 0, 0, 255];

/// Where a new stroke goes: the topmost visible, unlocked layer of the first scene, and the
/// drawing it shows at `frame` if it shows one.
pub fn find_target(project: &Project, frame: u32) -> Option<(SceneId, LayerId, Option<DrawingId>)> {
    let scene = project.scenes.first()?;
    let layer = scene
        .layers
        .iter()
        .rev()
        .find(|layer| layer.visible && !layer.locked)?;
    Some((scene.id, layer.id, layer.drawing_at(frame).map(|d| d.id)))
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
    ) -> (Self, Stroke) {
        let stroke = Stroke {
            id: StrokeId::new(),
            base_width: PEN_WIDTH,
            color: PEN_COLOR,
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
