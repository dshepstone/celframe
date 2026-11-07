//! 2D affine transforms for moving, scaling, rotating and flipping strokes.

use celframe_core::{Stroke, StrokeSample};

/// Maps `(x, y)` to `(a·x + c·y + e, b·x + d·y + f)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Affine {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub fn translate(dx: f64, dy: f64) -> Self {
        Self {
            e: dx,
            f: dy,
            ..Self::IDENTITY
        }
    }

    /// Scales by `sx` and `sy` about `centre`.
    pub fn scale_about(centre: [f64; 2], sx: f64, sy: f64) -> Self {
        Self {
            a: sx,
            d: sy,
            e: centre[0] * (1.0 - sx),
            f: centre[1] * (1.0 - sy),
            ..Self::IDENTITY
        }
    }

    /// Turns counter-clockwise by `degrees` about `centre`. Quarter turns are exact.
    pub fn rotate_about(centre: [f64; 2], degrees: f64) -> Self {
        let turns = degrees.rem_euclid(360.0);
        let (sin, cos) = match turns {
            0.0 => (0.0, 1.0),
            90.0 => (1.0, 0.0),
            180.0 => (0.0, -1.0),
            270.0 => (-1.0, 0.0),
            _ => turns.to_radians().sin_cos(),
        };
        let [x, y] = centre;
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: x - cos * x + sin * y,
            f: y - sin * x - cos * y,
        }
    }

    pub fn apply(&self, [x, y]: [f64; 2]) -> [f64; 2] {
        [
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        ]
    }

    /// The stroke with its centreline transformed. Widths are unchanged.
    pub fn apply_to_stroke(&self, stroke: &Stroke) -> Stroke {
        Stroke {
            samples: stroke
                .samples
                .iter()
                .map(|s| {
                    let [x, y] = self.apply([s.x, s.y]);
                    StrokeSample { x, y, ..*s }
                })
                .collect(),
            ..stroke.clone()
        }
    }
}

/// The smallest box holding every stroke's centreline, as `[min_x, min_y, max_x, max_y]`.
pub fn centreline_bounds<'a>(strokes: impl IntoIterator<Item = &'a Stroke>) -> Option<[f64; 4]> {
    strokes
        .into_iter()
        .flat_map(|s| &s.samples)
        .fold(None, |bounds, s| {
            Some(match bounds {
                None => [s.x, s.y, s.x, s.y],
                Some([x0, y0, x1, y1]) => [x0.min(s.x), y0.min(s.y), x1.max(s.x), y1.max(s.y)],
            })
        })
}
