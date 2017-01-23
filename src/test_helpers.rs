use num_traits::Float;

pub fn within_epsilon<F: Float>(x: F, y: F, epsilon: F) -> bool {
    let a = x.abs();
    let b = y.abs();
    let delta = (a - b).abs();

    if a.is_infinite() ||
        a.is_nan() ||
        b.is_infinite() ||
        b.is_nan() {
        false
    } else if a == b {
        true
    } else if (a == F::zero()) || (b == F::zero()) {
        delta <= epsilon
    } else {
        delta / b <= epsilon
    }
}

#[test]
fn within_epsilon_true_if_floats_equal() {
    assert!(within_epsilon(1.0, 1.0, 0.0001));
    assert!(within_epsilon(0.0, 0.0, 0.0001));
}

#[test]
fn within_epsilon_true_if_floats_close() {
    assert!(within_epsilon(1.00000001, 1.00000002, 0.01));
    assert!(within_epsilon(0.0, 0.00000002, 0.01));
    assert!(within_epsilon(0.00000001, 0.0, 0.01));
    assert!(within_epsilon(0.0000001, -0.0000001, 0.00001));
}

#[test]
fn within_epsilon_false_if_floats_far() {
    assert!(!within_epsilon(1.0, 10.0, 0.1));
    assert!(!within_epsilon(1.0, 0.0, 0.1));
    assert!(!within_epsilon(0.0, 1.0, 0.1));
}

#[test]
fn within_epsilon_false_if_floats_infinite_or_nan() {
    assert!(!within_epsilon(Float::infinity(), Float::neg_infinity(), 0.1));
    assert!(!within_epsilon(Float::infinity(), Float::infinity(), 0.1));
    assert!(!within_epsilon(Float::infinity(), 1.0, 0.1));
    assert!(!within_epsilon(Float::nan(), 1.0, 0.1));
    assert!(!within_epsilon(1.0, Float::nan(), 0.1));
}
