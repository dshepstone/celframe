use celframe_core::{Animated, Interpolation, Key};

fn key(frame: u32, value: f64, interpolation: Interpolation) -> Key {
    Key {
        frame,
        value,
        interpolation,
    }
}

#[test]
fn without_keys_the_base_value_holds() {
    assert_eq!(Animated::constant(0.4).at(100), 0.4);
}

#[test]
fn keys_interpolate_linearly_eased_or_stepped() {
    let mut value = Animated::constant(1.0);
    value.set_key(key(10, 0.0, Interpolation::Linear));
    value.set_key(key(20, 1.0, Interpolation::Ease));
    value.set_key(key(30, 0.0, Interpolation::Step));
    value.set_key(key(40, 0.5, Interpolation::Linear));
    assert_eq!(value.at(0), 0.0, "before the first key");
    assert_eq!(value.at(15), 0.5, "linear halfway");
    // Smoothstep at a quarter: 3(1/4)^2 - 2(1/4)^3 = 0.15625.
    assert!((value.at(22) - (1.0 - 0.104)).abs() < 1e-12);
    assert_eq!(value.at(25), 0.5, "ease halfway is still halfway");
    assert_eq!(value.at(39), 0.0, "step holds until the next key");
    assert_eq!(value.at(40), 0.5);
    assert_eq!(value.at(99), 0.5, "after the last key");
}

#[test]
fn setting_a_key_at_an_existing_frame_replaces_it() {
    let mut value = Animated::constant(1.0);
    assert!(value.set_key(key(5, 0.2, Interpolation::Ease)).is_none());
    let old = value.set_key(key(5, 0.7, Interpolation::Linear)).unwrap();
    assert_eq!(old.value, 0.2);
    assert_eq!(value.keys.len(), 1);
    assert_eq!(value.remove_key(5).unwrap().value, 0.7);
    assert!(value.remove_key(5).is_none());
}

#[test]
fn validity_needs_ordered_keys_and_values_in_range() {
    let mut value = Animated::constant(1.0);
    value.set_key(key(1, 0.5, Interpolation::Ease));
    assert!(value.is_valid(0.0..=1.0));
    value.keys.push(key(1, 0.2, Interpolation::Ease));
    assert!(!value.is_valid(0.0..=1.0), "two keys at one frame");
    assert!(!Animated::constant(2.0).is_valid(0.0..=1.0));
}
