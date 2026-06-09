fn check_up(x: f32) {
    let mut y = x;

    for n in 0..=300 {
        assert_eq!(metallic::ldexpf(x, n).to_bits(), y.to_bits());
        y *= 2.0;
    }
}

fn check_down(x: f32) {
    let mut coefficient = 0.5;

    for n in 1..=300 {
        #[allow(clippy::cast_possible_truncation)]
        let y = (f64::from(x) * coefficient) as f32;
        assert_eq!(metallic::ldexpf(x, -n).to_bits(), y.to_bits());
        coefficient *= 0.5;
    }
}

#[test]
fn test_finite_positive_up() {
    (0..f32::INFINITY.to_bits()).step_by(17).for_each(|i| {
        check_up(f32::from_bits(i));
    });
}

#[test]
fn test_finite_positive_down() {
    (0..f32::INFINITY.to_bits()).step_by(17).for_each(|i| {
        check_down(f32::from_bits(i));
    });
}

#[test]
fn test_finite_negative_up() {
    (0..f32::INFINITY.to_bits()).step_by(17).for_each(|i| {
        check_up(-f32::from_bits(i));
    });
}

#[test]
fn test_finite_negative_down() {
    (0..f32::INFINITY.to_bits()).step_by(17).for_each(|i| {
        check_down(-f32::from_bits(i));
    });
}

#[test]
fn test_infinite() {
    for n in -300..=300 {
        assert!(metallic::ldexpf(f32::INFINITY, n).eq(&f32::INFINITY));
        assert!(metallic::ldexpf(f32::NEG_INFINITY, n).eq(&f32::NEG_INFINITY));
    }

    (f32::INFINITY.to_bits() + 1..0x8000_0000).for_each(|i| {
        let x = f32::from_bits(i);

        for n in -300..=300 {
            assert!(metallic::ldexpf(x, n).is_nan());
            assert!(metallic::ldexpf(-x, n).is_nan());
        }
    });
}
