//! Geometric shapes as stroke centrelines. See `docs/specs/shape-tools.md`.

use std::f64::consts::{FRAC_PI_2, TAU};

use celframe_core::StrokeSample;

pub const POLYGON_SIDES: std::ops::RangeInclusive<u32> = 3..=12;
const LINE_SNAP: f64 = 15.0 * std::f64::consts::PI / 180.0;
const ELLIPSE_SEGMENTS: std::ops::RangeInclusive<usize> = 32..=256;
/// Ellipses get one segment per this many scene units of perimeter, within `ELLIPSE_SEGMENTS`.
const ELLIPSE_SEGMENT_LENGTH: f64 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Line,
    Rectangle,
    Ellipse,
    Polygon { sides: u32 },
}

/// How the modifier keys change a shape.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Constraints {
    /// Snap a line's angle, or make a box square.
    pub constrain: bool,
    /// Treat the start point as the middle of the shape.
    pub from_centre: bool,
}

/// The centreline of `shape` dragged from `start` to `end`, in scene space, at full pressure.
/// Returns None when the drag hasn't covered any distance yet, or a polygon has a side count
/// outside `POLYGON_SIDES`.
pub fn shape_samples(
    shape: Shape,
    start: [f64; 2],
    end: [f64; 2],
    constraints: Constraints,
) -> Option<Vec<StrokeSample>> {
    let mut delta = [end[0] - start[0], end[1] - start[1]];
    if delta == [0.0, 0.0] || !delta.iter().all(|d| d.is_finite()) {
        return None;
    }
    let points = if shape == Shape::Line {
        if constraints.constrain {
            let length = delta[0].hypot(delta[1]);
            let angle = (delta[1].atan2(delta[0]) / LINE_SNAP).round() * LINE_SNAP;
            delta = [length * angle.cos(), length * angle.sin()];
        }
        if constraints.from_centre {
            vec![
                [start[0] - delta[0], start[1] - delta[1]],
                [start[0] + delta[0], start[1] + delta[1]],
            ]
        } else {
            vec![start, [start[0] + delta[0], start[1] + delta[1]]]
        }
    } else {
        if constraints.constrain {
            let side = delta[0].abs().max(delta[1].abs());
            delta = [side.copysign(delta[0]), side.copysign(delta[1])];
        }
        let (centre, half) = if constraints.from_centre {
            (start, [delta[0].abs(), delta[1].abs()])
        } else {
            (
                [start[0] + delta[0] / 2.0, start[1] + delta[1] / 2.0],
                [delta[0].abs() / 2.0, delta[1].abs() / 2.0],
            )
        };
        closed_shape(shape, centre, half)?
    };
    Some(
        points
            .into_iter()
            .map(|[x, y]| StrokeSample {
                x,
                y,
                pressure: 1.0,
            })
            .collect(),
    )
}

/// Points around a closed shape fitting a box, ending where they start.
fn closed_shape(shape: Shape, centre: [f64; 2], half: [f64; 2]) -> Option<Vec<[f64; 2]>> {
    let at = |angle: f64| {
        [
            centre[0] + half[0] * angle.cos(),
            centre[1] + half[1] * angle.sin(),
        ]
    };
    let mut points = match shape {
        Shape::Line => unreachable!("lines aren't closed"),
        Shape::Rectangle => {
            let [x0, y0] = [centre[0] - half[0], centre[1] - half[1]];
            let [x1, y1] = [centre[0] + half[0], centre[1] + half[1]];
            vec![[x0, y1], [x1, y1], [x1, y0], [x0, y0]]
        }
        Shape::Ellipse => {
            // Ramanujan's approximation of the perimeter.
            let (a, b) = (half[0], half[1]);
            let perimeter =
                std::f64::consts::PI * (3.0 * (a + b) - ((3.0 * a + b) * (a + 3.0 * b)).sqrt());
            let segments = ((perimeter / ELLIPSE_SEGMENT_LENGTH).ceil() as usize)
                .clamp(*ELLIPSE_SEGMENTS.start(), *ELLIPSE_SEGMENTS.end());
            (0..segments)
                .map(|i| at(FRAC_PI_2 + TAU * i as f64 / segments as f64))
                .collect()
        }
        Shape::Polygon { sides } => {
            if !POLYGON_SIDES.contains(&sides) {
                return None;
            }
            (0..sides)
                .map(|i| at(FRAC_PI_2 + TAU * f64::from(i) / f64::from(sides)))
                .collect()
        }
    };
    points.push(points[0]);
    Some(points)
}
