//! Finding strokes under a point or inside a box. See `docs/specs/selection.md`.

use celframe_core::Stroke;

/// How far `point` is from the stroke's painted edge, in scene units. Negative inside the
/// stroke. Infinite for a stroke with no samples.
pub fn distance_to_stroke(stroke: &Stroke, point: [f64; 2]) -> f64 {
    let half = |pressure: f32| stroke.base_width * f64::from(pressure) / 2.0;
    match stroke.samples.as_slice() {
        [] => f64::INFINITY,
        [only] => (point[0] - only.x).hypot(point[1] - only.y) - half(only.pressure),
        samples => samples
            .windows(2)
            .map(|pair| {
                let (a, b) = (pair[0], pair[1]);
                let (distance, t) = to_segment(point, [a.x, a.y], [b.x, b.y]);
                let pressure = a.pressure + (b.pressure - a.pressure) * t as f32;
                distance - half(pressure)
            })
            .fold(f64::INFINITY, f64::min),
    }
}

/// The smallest box holding the stroke's painted area, as `[min_x, min_y, max_x, max_y]`.
/// None for a stroke with no samples.
pub fn stroke_bounds(stroke: &Stroke) -> Option<[f64; 4]> {
    stroke.samples.iter().fold(None, |bounds, s| {
        let half = stroke.base_width * f64::from(s.pressure) / 2.0;
        let point = [s.x - half, s.y - half, s.x + half, s.y + half];
        Some(match bounds {
            None => point,
            Some(b) => union(b, point),
        })
    })
}

pub fn union(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0].min(b[0]),
        a[1].min(b[1]),
        a[2].max(b[2]),
        a[3].max(b[3]),
    ]
}

/// True when any of the stroke's centreline lies inside the box `[min_x, min_y, max_x, max_y]`.
pub fn stroke_touches_box(stroke: &Stroke, area: [f64; 4]) -> bool {
    let inside = |x: f64, y: f64| x >= area[0] && x <= area[2] && y >= area[1] && y <= area[3];
    if stroke.samples.iter().any(|s| inside(s.x, s.y)) {
        return true;
    }
    // A segment can cross the box with both ends outside it.
    let corners = [
        [area[0], area[1]],
        [area[2], area[1]],
        [area[2], area[3]],
        [area[0], area[3]],
    ];
    stroke.samples.windows(2).any(|pair| {
        let (a, b) = ([pair[0].x, pair[0].y], [pair[1].x, pair[1].y]);
        (0..4).any(|i| segments_cross(a, b, corners[i], corners[(i + 1) % 4]))
    })
}

/// Distance from `p` to the segment, and how far along it the nearest point is, from 0 to 1.
fn to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> (f64, f64) {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let length_squared = dx * dx + dy * dy;
    let t = if length_squared > 0.0 {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / length_squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    ((p[0] - (a[0] + dx * t)).hypot(p[1] - (a[1] + dy * t)), t)
}

fn segments_cross(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let side = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
    };
    let (d1, d2) = (side(c, d, a), side(c, d, b));
    let (d3, d4) = (side(a, b, c), side(a, b, d));
    d1 * d2 <= 0.0 && d3 * d4 <= 0.0
}
