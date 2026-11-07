use std::f64::consts::PI;

use celframe_core::{Stroke, StrokeId, StrokeSample};
use celframe_geom::{StrokeMesh, tessellate_stroke};

const TOLERANCE: f32 = 0.01;

fn stroke(base_width: f64, samples: &[(f64, f64, f32)]) -> Stroke {
    Stroke {
        id: StrokeId::new(),
        base_width,
        color: [0, 0, 0, 255],
        art: celframe_core::Art::Line,
        samples: samples
            .iter()
            .map(|&(x, y, pressure)| StrokeSample { x, y, pressure })
            .collect(),
    }
}

fn mesh_of(stroke: &Stroke) -> StrokeMesh {
    let mut mesh = StrokeMesh::default();
    tessellate_stroke(stroke, TOLERANCE, &mut mesh);
    mesh
}

/// Total triangle area. The tessellator doesn't overlap triangles, so this is the shape's area.
fn area(mesh: &StrokeMesh) -> f64 {
    mesh.indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| mesh.positions[i as usize].map(f64::from));
            ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0
        })
        .sum()
}

fn bounds(mesh: &StrokeMesh) -> [f32; 4] {
    mesh.positions.iter().fold(
        [f32::MAX, f32::MAX, f32::MIN, f32::MIN],
        |[x0, y0, x1, y1], &[x, y]| [x0.min(x), y0.min(y), x1.max(x), y1.max(y)],
    )
}

fn assert_close(actual: f64, expected: f64, relative: f64) {
    assert!(
        (actual - expected).abs() <= expected.abs() * relative,
        "{actual} is not within {relative} of {expected}"
    );
}

fn assert_valid(mesh: &StrokeMesh) {
    assert_eq!(mesh.indices.len() % 3, 0);
    assert!(
        mesh.indices
            .iter()
            .all(|&i| (i as usize) < mesh.positions.len())
    );
    assert!(mesh.positions.iter().flatten().all(|v| v.is_finite()));
}

#[test]
fn an_empty_stroke_draws_nothing() {
    let mesh = mesh_of(&stroke(4.0, &[]));
    assert!(mesh.positions.is_empty() && mesh.indices.is_empty());
}

#[test]
fn a_stroke_with_no_width_draws_nothing() {
    let mesh = mesh_of(&stroke(4.0, &[(0.0, 0.0, 0.0), (10.0, 0.0, 0.0)]));
    assert_eq!(area(&mesh), 0.0);
}

#[test]
fn pressure_that_drops_to_zero_mid_stroke_still_gives_a_valid_mesh() {
    let mesh = mesh_of(&stroke(
        4.0,
        &[
            (0.0, 0.0, 1.0),
            (10.0, 0.0, 0.0),
            (20.0, 0.0, 0.0),
            (30.0, 0.0, 1.0),
        ],
    ));
    assert_valid(&mesh);
    assert!(area(&mesh) > 0.0);
}

#[test]
fn samples_in_one_place_are_a_dot() {
    let mesh = mesh_of(&stroke(4.0, &[(1.0, 1.0, 1.0), (1.0, 1.0, 0.5)]));
    assert_valid(&mesh);
    assert_close(area(&mesh), PI * 2.0 * 2.0, 0.02);
}

#[test]
fn a_single_sample_is_a_round_dot() {
    let mesh = mesh_of(&stroke(10.0, &[(3.0, 4.0, 0.5)]));
    assert_valid(&mesh);
    // Width 5, so radius 2.5.
    assert_close(area(&mesh), PI * 2.5 * 2.5, 0.02);
    let [x0, y0, x1, y1] = bounds(&mesh);
    assert_close(f64::from(x1 - x0), 5.0, 0.01);
    assert_close(f64::from(y1 - y0), 5.0, 0.01);
    assert_close(f64::from(x0 + x1) / 2.0, 3.0, 0.01);
}

#[test]
fn a_straight_stroke_is_a_band_with_round_ends() {
    let mesh = mesh_of(&stroke(
        4.0,
        &[(0.0, 0.0, 1.0), (50.0, 0.0, 1.0), (100.0, 0.0, 1.0)],
    ));
    assert_valid(&mesh);
    // A 100 × 4 band plus two half-circles of radius 2.
    assert_close(area(&mesh), 100.0 * 4.0 + PI * 4.0, 0.01);
    let [x0, y0, x1, y1] = bounds(&mesh);
    assert_close(f64::from(x0), -2.0, 0.01);
    assert_close(f64::from(x1), 102.0, 0.01);
    assert_close(f64::from(y1 - y0), 4.0, 0.01);
}

#[test]
fn width_follows_pressure() {
    // Pressure rises from 0.25 to 1 along the stroke, so it gets wider.
    let samples: Vec<_> = (0..=10)
        .map(|i| (f64::from(i) * 10.0, 0.0, 0.25 + 0.075 * i as f32))
        .collect();
    let mesh = mesh_of(&stroke(8.0, &samples));
    assert_valid(&mesh);
    let half_width_near = |x: f32| {
        mesh.positions
            .iter()
            .filter(|p| (p[0] - x).abs() < 0.5)
            .map(|p| p[1].abs())
            .fold(0.0_f32, f32::max)
    };
    // Width 8 × 0.4 = 3.2 at x = 20, and 8 × 0.85 = 6.8 at x = 80.
    assert_close(f64::from(half_width_near(20.0)), 1.6, 0.05);
    assert_close(f64::from(half_width_near(80.0)), 3.4, 0.05);
}

#[test]
fn sharp_turns_stay_inside_the_round_join() {
    let mesh = mesh_of(&stroke(
        4.0,
        &[(0.0, 0.0, 1.0), (50.0, 0.0, 1.0), (0.0, 1.0, 1.0)],
    ));
    assert_valid(&mesh);
    // A mitre join would spike far past x = 50. A round join stays within the half width.
    assert!(bounds(&mesh)[2] <= 52.0 + TOLERANCE);
}

#[test]
fn strokes_append_into_one_mesh() {
    let a = stroke(2.0, &[(0.0, 0.0, 1.0), (10.0, 0.0, 1.0)]);
    let b = stroke(2.0, &[(0.0, 20.0, 1.0), (10.0, 20.0, 1.0)]);
    let (mesh_a, mesh_b) = (mesh_of(&a), mesh_of(&b));

    let mut both = StrokeMesh::default();
    tessellate_stroke(&a, TOLERANCE, &mut both);
    tessellate_stroke(&b, TOLERANCE, &mut both);
    assert_valid(&both);
    assert_eq!(
        both.positions.len(),
        mesh_a.positions.len() + mesh_b.positions.len()
    );
    assert_close(area(&both), area(&mesh_a) + area(&mesh_b), 1e-9);
}

#[test]
fn the_mesh_is_the_same_every_time() {
    let s = stroke(3.0, &[(0.0, 0.0, 0.3), (5.0, 7.0, 0.9), (9.0, -2.0, 0.6)]);
    assert_eq!(mesh_of(&s), mesh_of(&s));
}
