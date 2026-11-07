use celframe_core::StrokeSample;
use celframe_geom::{Erased, erase};

fn line(xs: &[f64]) -> Vec<StrokeSample> {
    xs.iter()
        .map(|&x| StrokeSample {
            x,
            y: 0.0,
            pressure: 0.5,
        })
        .collect()
}

fn pieces(result: Erased) -> Vec<Vec<StrokeSample>> {
    match result {
        Erased::Pieces(pieces) => pieces,
        Erased::Untouched => panic!("expected the stroke to be cut"),
    }
}

#[test]
fn an_eraser_that_misses_leaves_the_stroke_alone() {
    let stroke = line(&[0.0, 100.0]);
    assert_eq!(erase(&stroke, &[[50.0, 30.0]], 10.0), Erased::Untouched);
    assert_eq!(erase(&stroke, &[], 10.0), Erased::Untouched);
    assert_eq!(erase(&stroke, &[[50.0, 0.0]], 0.0), Erased::Untouched);
}

#[test]
fn a_dab_in_the_middle_of_a_long_segment_cuts_it_in_two() {
    // Neither sample is near the eraser, but the segment between them is.
    let result = pieces(erase(&line(&[0.0, 100.0]), &[[50.0, 0.0]], 10.0));
    assert_eq!(result.len(), 2);
    let (left, right) = (&result[0], &result[1]);
    assert_eq!(left[0].x, 0.0);
    assert_eq!(right.last().unwrap().x, 100.0);
    let left_end = left.last().unwrap().x;
    let right_start = right[0].x;
    assert!((37.5..40.0).contains(&left_end), "cut at {left_end}");
    assert!((60.0..=62.5).contains(&right_start), "cut at {right_start}");
    assert!(result.iter().flatten().all(|s| s.pressure == 0.5));
}

#[test]
fn a_drag_across_the_whole_stroke_erases_all_of_it() {
    let stroke = line(&[0.0, 10.0, 20.0]);
    let result = pieces(erase(&stroke, &[[-5.0, 0.0], [25.0, 0.0]], 2.0));
    assert!(result.is_empty());
}

#[test]
fn erasing_an_end_leaves_one_piece() {
    let result = pieces(erase(&line(&[0.0, 10.0, 20.0, 30.0]), &[[30.0, 0.0]], 5.0));
    assert_eq!(result.len(), 1);
    assert_eq!(result[0][0].x, 0.0);
    assert!(result[0].last().unwrap().x <= 25.0);
}

#[test]
fn a_dot_is_erased_whole_or_not_at_all() {
    let dot = line(&[0.0]);
    assert_eq!(
        pieces(erase(&dot, &[[1.0, 0.0]], 2.0)),
        Vec::<Vec<_>>::new()
    );
    assert_eq!(erase(&dot, &[[5.0, 0.0]], 2.0), Erased::Untouched);
}

#[test]
fn pressure_is_interpolated_across_new_samples() {
    let mut stroke = line(&[0.0, 100.0]);
    stroke[1].pressure = 1.0;
    let result = pieces(erase(&stroke, &[[100.0, 0.0]], 10.0));
    let end = *result[0].last().unwrap();
    let expected = 0.5 + 0.5 * (end.x / 100.0) as f32;
    assert!((end.pressure - expected).abs() < 1e-6);
}
