//! Strokes, Bezier curves and smoothing.

mod erase;
mod shapes;
mod smooth;
mod tessellate;

pub use erase::{Erased, erase};
pub use shapes::{Constraints, POLYGON_SIDES, Shape, shape_samples};
pub use smooth::{StrokeSmoother, smooth};
pub use tessellate::{StrokeMesh, tessellate_stroke};
