use celframe_core::{Art, Stroke, StrokeId, StrokeSample};
use celframe_geom::{distance_to_stroke, stroke_bounds, stroke_touches_box};

fn stroke(width: f64, points: &[(f64, f64)]) -> Stroke {
    Stroke {
        id: StrokeId::new(),
        base_width: width,
        color: [0, 0, 0, 255],
        art: Art::Line,
        samples: points
            .iter()
            .map(|&(x, y)| StrokeSample {
                x,
                y,
                pressure: 1.0,
            })
            .collect(),
    }
}

#[test]
fn distance_is_measured_to_the_painted_edge() {
    let line = stroke(4.0, &[(0.0, 0.0), (10.0, 0.0)]);
    assert_eq!(distance_to_stroke(&line, [5.0, 5.0]), 3.0);
    assert!(
        distance_to_stroke(&line, [5.0, 1.0]) < 0.0,
        "inside the stroke"
    );
    assert_eq!(
        distance_to_stroke(&line, [13.0, 4.0]),
        3.0,
        "past the round end"
    );
    let dot = stroke(2.0, &[(0.0, 0.0)]);
    assert_eq!(distance_to_stroke(&dot, [3.0, 0.0]), 2.0);
    assert_eq!(
        distance_to_stroke(&stroke(1.0, &[]), [0.0, 0.0]),
        f64::INFINITY
    );
}

#[test]
fn bounds_include_the_stroke_width() {
    let line = stroke(4.0, &[(0.0, 0.0), (10.0, 5.0)]);
    assert_eq!(stroke_bounds(&line), Some([-2.0, -2.0, 12.0, 7.0]));
    assert_eq!(stroke_bounds(&stroke(1.0, &[])), None);
}

#[test]
fn a_box_selects_strokes_with_any_part_inside_it() {
    let line = stroke(1.0, &[(0.0, 0.0), (10.0, 0.0)]);
    assert!(
        stroke_touches_box(&line, [8.0, -1.0, 20.0, 1.0]),
        "an end inside"
    );
    assert!(
        stroke_touches_box(&line, [4.0, -1.0, 6.0, 1.0]),
        "crossing the box"
    );
    assert!(
        !stroke_touches_box(&line, [4.0, 1.0, 6.0, 3.0]),
        "beside the box"
    );
}

#[test]
fn a_lasso_selects_strokes_with_any_part_inside_it() {
    use celframe_geom::{inside_polygon, stroke_touches_lasso};
    // A triangle with its point at the top.
    let lasso = [[0.0, 0.0], [10.0, 0.0], [5.0, 10.0]];
    assert!(inside_polygon([5.0, 2.0], &lasso));
    assert!(!inside_polygon([1.0, 8.0], &lasso));

    let through = stroke(1.0, &[(-5.0, 2.0), (15.0, 2.0)]);
    assert!(
        stroke_touches_lasso(&through, &lasso),
        "crosses with both ends outside"
    );
    let beside = stroke(1.0, &[(0.0, 9.0), (1.0, 9.0)]);
    assert!(!stroke_touches_lasso(&beside, &lasso));
    let inside = stroke(1.0, &[(5.0, 3.0)]);
    assert!(stroke_touches_lasso(&inside, &lasso));
    assert!(
        !stroke_touches_lasso(&inside, &lasso[..2]),
        "two points enclose nothing"
    );
}
