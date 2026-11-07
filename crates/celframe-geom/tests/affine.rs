use celframe_geom::Affine;

#[test]
fn quarter_turns_are_exact() {
    let turn = Affine::rotate_about([10.0, 0.0], 90.0);
    assert_eq!(turn.apply([20.0, 0.0]), [10.0, 10.0]);
    let back = Affine::rotate_about([10.0, 0.0], -90.0);
    assert_eq!(back.apply([10.0, 10.0]), [20.0, 0.0]);
    assert_eq!(Affine::rotate_about([3.0, 4.0], 360.0), Affine::IDENTITY);
}

#[test]
fn any_angle_turns_about_the_centre() {
    let turn = Affine::rotate_about([1.0, 1.0], 45.0);
    let [x, y] = turn.apply([2.0, 1.0]);
    let h = std::f64::consts::FRAC_1_SQRT_2;
    assert!((x - (1.0 + h)).abs() < 1e-12 && (y - (1.0 + h)).abs() < 1e-12);
    // Sine and cosine round differently on each platform, so allow a tiny error.
    let [cx, cy] = turn.apply([1.0, 1.0]);
    assert!(
        (cx - 1.0).abs() < 1e-12 && (cy - 1.0).abs() < 1e-12,
        "the centre stays put"
    );
}

#[test]
fn scaling_keeps_the_centre_and_negative_scale_flips() {
    let scale = Affine::scale_about([5.0, 5.0], 2.0, 0.5);
    assert_eq!(scale.apply([5.0, 5.0]), [5.0, 5.0]);
    assert_eq!(scale.apply([6.0, 7.0]), [7.0, 6.0]);
    assert_eq!(
        Affine::scale_about([5.0, 0.0], -1.0, 1.0).apply([7.0, 3.0]),
        [3.0, 3.0]
    );
    assert_eq!(Affine::translate(1.0, -2.0).apply([0.0, 0.0]), [1.0, -2.0]);
}
