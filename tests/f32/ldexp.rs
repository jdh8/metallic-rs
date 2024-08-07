use metallic::f32 as metal;

fn up(x: f32) {
    let mut y = x;

    for n in 0..300 {
        assert_eq!(metal::ldexp(x, n).to_bits(), y.to_bits());
        y *= 2.0;
    }
}

fn down(x: f32) {
    let mut coefficient = 1.0;

    for n in 0..300 {
        #[allow(clippy::cast_possible_truncation)]
        let y = (f64::from(x) * coefficient) as f32;
        assert_eq!(metal::ldexp(x, -n).to_bits(), y.to_bits());
        coefficient *= 0.5;
    }
}

fn run(x: f32) {
    up(x);
    up(-x);
    down(x);
    down(-x);
}

#[test]
fn test_ldexp() {
    run(f32::INFINITY);

    (0..0x7F00_0000).step_by(7146).for_each(|i| {
        run(f32::from_bits(i));
    });

    (0x7FC0_0000..0x8000_0000).step_by(9876).for_each(|i| {
        let x = f32::from_bits(i);
        assert!(metal::ldexp(x, 7).is_nan());
        assert!(metal::ldexp(x, -7).is_nan());
    });
}
