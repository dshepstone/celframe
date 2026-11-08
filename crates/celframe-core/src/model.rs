//! The document model. Units are defined in `docs/specs/units-and-coordinates.md`.

use serde::{Deserialize, Serialize};

use crate::animated::Animated;
use crate::camera::Camera;
use crate::ids::{DrawingId, LayerId, ProjectId, SceneId, StrokeId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub scenes: Vec<Scene>,
}

impl Project {
    /// A new project as the app creates it: like `new`, but the first layer already shows an
    /// empty drawing at frame 0, so the first stroke is a single undoable edit.
    pub fn starter(name: impl Into<String>) -> Self {
        let mut project = Self::new(name);
        let layer = &mut project.scenes[0].layers[0];
        let drawing = Drawing::new();
        let exposure = Exposure::new(0, 1, drawing.id);
        layer.drawings.push(drawing);
        layer
            .insert_exposure(exposure)
            .expect("a new layer has room for one exposure");
        project
    }

    /// Creates a project with one default scene.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: ProjectId::new(),
            name: name.into(),
            scenes: vec![Scene::new("Scene 1")],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub id: SceneId,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate: u32,
    /// Bottom layer first.
    pub layers: Vec<Layer>,
    pub camera: Camera,
}

impl Scene {
    /// Strokes shown at a frame, bottom first: each visible layer's exposed drawing, in layer
    /// order, with each drawing's colour art below its line art. Hidden layers and layers with
    /// nothing exposed draw nothing.
    pub fn visible_strokes(&self, frame: u32) -> impl Iterator<Item = &Stroke> {
        self.layers
            .iter()
            .filter(|layer| layer.visible)
            .filter_map(move |layer| layer.drawing_at(frame))
            .flat_map(Drawing::strokes_in_draw_order)
    }

    /// Creates a 1920 × 1080, 24 fps scene with one empty layer.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: SceneId::new(),
            name: name.into(),
            width: 1920,
            height: 1080,
            frame_rate: 24,
            layers: vec![Layer::new("Layer 1")],
            camera: Camera::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub drawings: Vec<Drawing>,
    /// Sorted by start frame, with no overlaps.
    pub exposures: Vec<Exposure>,
    /// From 0 to 1, multiplying everything the layer draws. Keyable.
    pub opacity: Animated,
}

impl Layer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            visible: true,
            locked: false,
            drawings: Vec::new(),
            exposures: Vec::new(),
            opacity: Animated::constant(1.0),
        }
    }

    /// Inserts an exposure, keeping exposures sorted and free of overlaps.
    pub fn insert_exposure(&mut self, exposure: Exposure) -> Result<(), ExposureError> {
        if exposure.length == 0 {
            return Err(ExposureError::ZeroLength);
        }
        if !self.drawings.iter().any(|d| d.id == exposure.drawing) {
            return Err(ExposureError::UnknownDrawing);
        }
        if self.exposures.iter().any(|e| e.overlaps(&exposure)) {
            return Err(ExposureError::Overlap);
        }
        let index = self
            .exposures
            .partition_point(|e| e.start_frame < exposure.start_frame);
        self.exposures.insert(index, exposure);
        Ok(())
    }

    /// Returns the drawing exposed at a frame, if any.
    pub fn drawing_at(&self, frame: u32) -> Option<&Drawing> {
        let exposure = self.exposures.iter().find(|e| e.covers(frame))?;
        self.drawings.iter().find(|d| d.id == exposure.drawing)
    }
}

/// Holds one drawing from a start frame for a number of frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exposure {
    pub start_frame: u32,
    pub length: u32,
    pub drawing: DrawingId,
}

impl Exposure {
    pub const fn new(start_frame: u32, length: u32, drawing: DrawingId) -> Self {
        Self {
            start_frame,
            length,
            drawing,
        }
    }

    // u64 so an exposure near the last frame number can't overflow.
    fn end(&self) -> u64 {
        u64::from(self.start_frame) + u64::from(self.length)
    }

    fn covers(&self, frame: u32) -> bool {
        frame >= self.start_frame && u64::from(frame) < self.end()
    }

    fn overlaps(&self, other: &Exposure) -> bool {
        u64::from(self.start_frame) < other.end() && u64::from(other.start_frame) < self.end()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExposureError {
    ZeroLength,
    Overlap,
    UnknownDrawing,
}

impl std::fmt::Display for ExposureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ZeroLength => "an exposure must last at least one frame",
            Self::Overlap => "an exposure can't overlap another exposure",
            Self::UnknownDrawing => "an exposure must use a drawing on the same layer",
        })
    }
}

impl std::error::Error for ExposureError {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Drawing {
    pub id: DrawingId,
    /// Bottom stroke first.
    pub strokes: Vec<Stroke>,
}

impl Drawing {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            id: DrawingId::new(),
            strokes: Vec::new(),
        }
    }

    /// Strokes bottom first: all colour art, then all line art, each part in list order.
    pub fn strokes_in_draw_order(&self) -> impl Iterator<Item = &Stroke> {
        let part = |art| self.strokes.iter().filter(move |s| s.art == art);
        part(Art::Colour).chain(part(Art::Line))
    }
}

/// Which part of a drawing a stroke belongs to. See `docs/specs/line-and-colour-art.md`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Art {
    #[default]
    Line,
    Colour,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stroke {
    pub id: StrokeId,
    /// Width in scene units at full pressure.
    pub base_width: f64,
    /// Straight RGBA, 8 bits per channel.
    pub color: [u8; 4],
    pub art: Art,
    pub samples: Vec<StrokeSample>,
}

/// One point on a stroke's centreline, in scene space.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrokeSample {
    pub x: f64,
    pub y: f64,
    /// From 0.0 to 1.0. Input without pressure uses 1.0.
    pub pressure: f32,
}
