//! Cutting strokes with the eraser. See `docs/specs/eraser.md`.

use celframe_core::StrokeSample;

/// No segment is split into more pieces than this, however small the eraser.
const MAX_PIECES_PER_SEGMENT: f64 = 4096.0;

/// What erasing did to one stroke.
#[derive(Clone, Debug, PartialEq)]
pub enum Erased {
    /// The eraser didn't reach the stroke.
    Untouched,
    /// The parts of the stroke that remain, in order. Empty when all of it was erased.
    Pieces(Vec<Vec<StrokeSample>>),
}

/// Erases where a stroke's centreline passes within `radius` of the eraser's path. `path` is
/// the eraser's centre, as one point or a line through several points.
///
/// Near the eraser, segments are split into steps of at most a quarter of the radius, so a cut
/// lands within that distance of the eraser's edge. Remaining pieces of a single sample are
/// dropped, because they would show as dots.
pub fn erase(samples: &[StrokeSample], path: &[[f64; 2]], radius: f64) -> Erased {
    if path.is_empty() || samples.is_empty() || radius.is_nan() || radius <= 0.0 {
        return Erased::Untouched;
    }
    let near = |x: f64, y: f64, reach: f64| distance_to_path([x, y], path) <= radius + reach;
    if samples.len() == 1 {
        let s = samples[0];
        return if near(s.x, s.y, 0.0) {
            Erased::Pieces(Vec::new())
        } else {
            Erased::Untouched
        };
    }

    let step = radius / 4.0;
    let mut dense = vec![samples[0]];
    for pair in samples.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let length = (b.x - a.x).hypot(b.y - a.y);
        // Every point on the segment is within half its length of the middle.
        let middle = [(a.x + b.x) / 2.0, (a.y + b.y) / 2.0];
        let pieces = if near(middle[0], middle[1], length / 2.0) {
            (length / step).ceil().clamp(1.0, MAX_PIECES_PER_SEGMENT) as usize
        } else {
            1
        };
        for i in 1..pieces {
            let t = i as f64 / pieces as f64;
            dense.push(StrokeSample {
                x: a.x + (b.x - a.x) * t,
                y: a.y + (b.y - a.y) * t,
                pressure: a.pressure + (b.pressure - a.pressure) * t as f32,
            });
        }
        dense.push(b);
    }

    let erased: Vec<bool> = dense.iter().map(|s| near(s.x, s.y, 0.0)).collect();
    if !erased.contains(&true) {
        return Erased::Untouched;
    }
    let pieces = dense
        .iter()
        .zip(&erased)
        .collect::<Vec<_>>()
        .split(|(_, gone)| **gone)
        .filter(|run| run.len() >= 2)
        .map(|run| run.iter().map(|(s, _)| **s).collect())
        .collect();
    Erased::Pieces(pieces)
}

fn distance_to_path(point: [f64; 2], path: &[[f64; 2]]) -> f64 {
    if path.len() == 1 {
        return (point[0] - path[0][0]).hypot(point[1] - path[0][1]);
    }
    path.windows(2)
        .map(|pair| distance_to_segment(point, pair[0], pair[1]))
        .fold(f64::INFINITY, f64::min)
}

fn distance_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let length_squared = dx * dx + dy * dy;
    let t = if length_squared > 0.0 {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / length_squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (p[0] - (a[0] + dx * t)).hypot(p[1] - (a[1] + dy * t))
}
