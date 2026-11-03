use celframe_core::StrokeSample;
use celframe_geom::{StrokeSmoother, smooth};

fn s(x: f64, y: f64, pressure: f32) -> StrokeSample {
    StrokeSample { x, y, pressure }
}

fn jittery_line() -> Vec<StrokeSample> {
    (0..50)
        .map(|i| {
            let jitter = if i % 2 == 0 { 1.0 } else { -1.0 };
            s(f64::from(i) * 4.0, jitter, 0.5)
        })
        .collect()
}

#[test]
fn empty_and_single_samples_pass_through() {
    assert!(smooth(&[], 0.0).is_empty());
    assert_eq!(smooth(&[s(1.0, 2.0, 0.5)], 0.0), [s(1.0, 2.0, 0.5)]);
}

#[test]
fn both_ends_are_kept_exactly() {
    let input = jittery_line();
    let output = smooth(&input, 0.0);
    assert_eq!(output.len(), input.len());
    assert_eq!(output[0], input[0]);
    assert_eq!(output[output.len() - 1], input[input.len() - 1]);
}

#[test]
fn jitter_is_reduced() {
    let input = jittery_line();
    let output = smooth(&input, 0.0);
    let middle = &output[10..40];
    let worst = middle.iter().map(|p| p.y.abs()).fold(0.0, f64::max);
    assert!(worst < 0.1, "still jittery: {worst}");
}

#[test]
fn a_straight_evenly_spaced_line_does_not_move() {
    let input: Vec<_> = (0..10).map(|i| s(f64::from(i), 0.0, 1.0)).collect();
    assert_eq!(smooth(&input, 0.0), input);
}

#[test]
fn pressure_is_smoothed_too() {
    let input = [
        s(0.0, 0.0, 0.0),
        s(1.0, 0.0, 0.0),
        s(2.0, 0.0, 1.0),
        s(3.0, 0.0, 0.0),
        s(4.0, 0.0, 0.0),
    ];
    let output = smooth(&input, 0.0);
    assert!(output[2].pressure > 0.0 && output[2].pressure < 1.0);
    assert!(output[1].pressure > 0.0);
}

#[test]
fn nearby_samples_are_dropped() {
    let input = [s(0.0, 0.0, 1.0), s(0.1, 0.0, 1.0), s(5.0, 0.0, 1.0)];
    assert_eq!(smooth(&input, 1.0).len(), 2);
}

#[test]
fn invalid_samples_are_ignored_and_pressure_is_clamped() {
    let input = [
        s(0.0, 0.0, 2.0),
        s(f64::NAN, 0.0, 1.0),
        s(0.0, f64::INFINITY, 1.0),
        s(1.0, 0.0, f32::NAN),
        s(5.0, 0.0, -1.0),
    ];
    let output = smooth(&input, 0.0);
    assert_eq!(output.len(), 2);
    assert_eq!(output[0].pressure, 1.0);
    assert_eq!(output[1].pressure, 0.0);
}

#[test]
fn streaming_matches_batch_exactly() {
    let input = jittery_line();
    let mut smoother = StrokeSmoother::new(0.0);
    let mut streamed = Vec::new();
    for sample in &input {
        streamed.extend(smoother.push(*sample));
    }
    streamed.extend(smoother.finish());
    assert_eq!(streamed, smooth(&input, 0.0));
}

#[test]
fn smoothing_is_repeatable() {
    let input = jittery_line();
    assert_eq!(smooth(&input, 0.5), smooth(&input, 0.5));
}
