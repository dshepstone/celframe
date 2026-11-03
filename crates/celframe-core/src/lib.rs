//! Document model, commands and undo/redo.

mod ids;
mod model;

pub use ids::{DrawingId, LayerId, ProjectId, SceneId, StrokeId};
pub use model::{Drawing, Exposure, ExposureError, Layer, Project, Scene, Stroke, StrokeSample};

mod schema;
pub use schema::{SCHEMA_VERSION, SchemaCompatibility, schema_compatibility};
