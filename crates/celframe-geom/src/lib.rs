//! Strokes, Bezier curves and smoothing.

mod erase;
mod smooth;
mod tessellate;

pub use erase::{Erased, erase};
pub use smooth::{StrokeSmoother, smooth};
pub use tessellate::{StrokeMesh, tessellate_stroke};
