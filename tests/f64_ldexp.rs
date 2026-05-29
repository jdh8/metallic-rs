use metallic::f64 as metal;

/// `ldexp` is exact scaling by a power of two; compare bit-for-bit against `libm`.
#[test]
fn finite() {
    let xs = (0..f64::INFINITY.to_bits()).step_by((1 << 50) + 1);

    for i in xs {
        let x = f64::from_bits(i);

        for n in (-1100..=1100).step_by(11) {
            assert_eq!(
                metal::ldexp(x, n).to_bits(),
                libm::ldexp(x, n).to_bits(),
                "ldexp({x:e}, {n})"
            );
            assert_eq!(
                metal::ldexp(-x, n).to_bits(),
                libm::ldexp(-x, n).to_bits(),
                "ldexp({:e}, {n})",
                -x
            );
        }
    }
}

#[test]
fn nonfinite() {
    for n in -1100..=1100 {
        assert!(metal::ldexp(f64::INFINITY, n).eq(&f64::INFINITY));
        assert!(metal::ldexp(f64::NEG_INFINITY, n).eq(&f64::NEG_INFINITY));
        assert!(metal::ldexp(f64::NAN, n).is_nan());
        assert!(metal::ldexp(0.0, n).eq(&0.0));
        assert!(metal::ldexp(-0.0, n).to_bits() == (-0.0_f64).to_bits());
    }
}
