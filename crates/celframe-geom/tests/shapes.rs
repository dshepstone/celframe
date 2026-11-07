use celframe_geom::{Constraints, Shape, shape_samples};

fn points(shape: Shape, start: [f64; 2], end: [f64; 2], c: Constraints) -> Vec<[f64; 2]> {
    shape_samples(shape, start, end, c)
        .unwrap()
        .iter()
        .map(|s| {
            assert_eq!(s.pressure, 1.0);
            [s.x, s.y]
        })
        .collect()
}

fn close(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
}

const NONE: Constraints = Constraints {
    constrain: false,
    from_centre: false,
};
const SHIFT: Constraints = Constraints {
    constrain: true,
    from_centre: false,
};
const ALT: Constraints = Constraints {
    constrain: false,
    from_centre: true,
};

#[test]
fn a_drag_with_no_distance_draws_nothing() {
    assert!(shape_samples(Shape::Line, [1.0, 1.0], [1.0, 1.0], NONE).is_none());
    assert!(shape_samples(Shape::Ellipse, [0.0, 0.0], [f64::NAN, 1.0], NONE).is_none());
}

#[test]
fn a_line_goes_from_start_to_end_or_through_the_start_with_alt() {
    assert_eq!(
        points(Shape::Line, [0.0, 0.0], [3.0, 4.0], NONE),
        [[0.0, 0.0], [3.0, 4.0]]
    );
    assert_eq!(
        points(Shape::Line, [0.0, 0.0], [3.0, 4.0], ALT),
        [[-3.0, -4.0], [3.0, 4.0]]
    );
}

#[test]
fn shift_snaps_a_line_to_15_degree_steps_keeping_its_length() {
    let p = points(Shape::Line, [0.0, 0.0], [10.0, 1.0], SHIFT);
    assert!(close(p[1], [10.0f64.hypot(1.0), 0.0]), "{p:?}");
    let p = points(Shape::Line, [0.0, 0.0], [5.0, 5.2], SHIFT);
    assert!((p[1][0] - p[1][1]).abs() < 1e-9, "snapped to 45°: {p:?}");
}

#[test]
fn a_rectangle_has_four_corners_and_closes() {
    let p = points(Shape::Rectangle, [0.0, 0.0], [4.0, 2.0], NONE);
    assert_eq!(p.len(), 5);
    assert_eq!(p[0], p[4]);
    for corner in [[0.0, 0.0], [4.0, 0.0], [4.0, 2.0], [0.0, 2.0]] {
        assert!(p.contains(&corner), "missing {corner:?} in {p:?}");
    }
}

#[test]
fn shift_makes_a_square_and_alt_draws_from_the_centre() {
    let p = points(Shape::Rectangle, [0.0, 0.0], [4.0, -2.0], SHIFT);
    for corner in [[0.0, 0.0], [4.0, 0.0], [4.0, -4.0], [0.0, -4.0]] {
        assert!(p.contains(&corner), "missing {corner:?} in {p:?}");
    }
    let p = points(Shape::Rectangle, [0.0, 0.0], [4.0, 2.0], ALT);
    for corner in [[-4.0, -2.0], [4.0, 2.0]] {
        assert!(p.contains(&corner), "missing {corner:?} in {p:?}");
    }
}

#[test]
fn an_ellipse_fits_its_box_and_gets_more_segments_when_larger() {
    let small = points(Shape::Ellipse, [-10.0, -5.0], [10.0, 5.0], NONE);
    assert_eq!(small.len(), 33, "at least 32 segments, closed");
    assert!(close(small[0], [0.0, 5.0]), "starts at the top");
    for [x, y] in &small {
        assert!(((x / 10.0).powi(2) + (y / 5.0).powi(2) - 1.0).abs() < 1e-9);
    }
    let large = points(Shape::Ellipse, [-200.0, -200.0], [200.0, 200.0], NONE);
    assert!(large.len() > small.len() && large.len() <= 257);
}

#[test]
fn a_polygon_has_its_sides_with_a_corner_at_the_top() {
    let p = points(Shape::Polygon { sides: 5 }, [-1.0, -1.0], [1.0, 1.0], NONE);
    assert_eq!(p.len(), 6);
    assert!(close(p[0], [0.0, 1.0]));
    assert!(shape_samples(Shape::Polygon { sides: 2 }, [0.0, 0.0], [1.0, 1.0], NONE).is_none());
    assert!(shape_samples(Shape::Polygon { sides: 13 }, [0.0, 0.0], [1.0, 1.0], NONE).is_none());
}
