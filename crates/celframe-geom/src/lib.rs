//! Strokes, Bezier curves and smoothing.

mod smooth;
mod tessellate;

pub use smooth::{StrokeSmoother, smooth};
pub use tessellate::{StrokeMesh, tessellate_stroke};
