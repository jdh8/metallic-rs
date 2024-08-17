mod ldexp;
use core::num::FpCategory;
use metallic::f32 as metal;

/// Semantic identity like `Object.is` in JavaScript
///
/// This function works around comparison issues with NaNs and signed zeros.
/// To be specific, `is(f32::NAN, f32::NAN)` but not `is(0.0, -0.0)`.
fn is(x: f32, y: f32) -> bool {
    x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan())
}

/// Check if `f` returns the same result as `g` for every `f32` values
///
/// By "same result", I mean semantic identity as defined by [`is`].
fn test_identity(f: impl Fn(f32) -> f32, g: impl Fn(f32) -> f32) {
    let count = (0..=u32::MAX)
        .filter_map(|i| {
            let x = f32::from_bits(i);
            (!is(f(x), g(x))).then(|| {
                println!("{x:.9e}: {:.9e} != {:.9e}", f(x), g(x));
                Some(())
            })
        })
        .count();

    assert_eq!(count, 0, "There are {count} mismatches");
}

#[test]
fn test_round() {
    test_identity(metal::round, f32::round);
}

#[test]
fn test_cbrt() {
    test_identity(metal::cbrt, core_math::cbrtf);
}

#[test]
fn test_exp() {
    test_identity(metal::exp, core_math::expf);
}

#[test]
fn test_exp2() {
    test_identity(metal::exp2, core_math::exp2f);
}

#[test]
fn test_exp10() {
    test_identity(metal::exp10, core_math::exp10f);
}

#[test]
fn test_exp_m1() {
    test_identity(metal::exp_m1, core_math::expm1f);
}

#[test]
fn test_ln() {
    test_identity(metal::ln, core_math::logf);
}

#[test]
fn test_ln_1p() {
    test_identity(metal::ln_1p, core_math::log1pf);
}

#[test]
fn test_log2() {
    test_identity(metal::log2, core_math::log2f);
}

#[test]
fn test_log10() {
    test_identity(metal::log10, core_math::log10f);
}

#[test]
fn test_acosh() {
    test_identity(metal::acosh, core_math::acoshf);
}

#[test]
fn test_asinh() {
    test_identity(metal::asinh, core_math::asinhf);
}

#[test]
fn test_atanh() {
    test_identity(metal::atanh, core_math::atanhf);
}

#[test]
fn frexp() {
    (0..u32::MAX).for_each(|i| {
        let x = f32::from_bits(i);
        let (significand, exponent) = metal::frexp(x);

        match x.classify() {
            FpCategory::Nan => assert!(significand.is_nan()),
            FpCategory::Infinite => assert_eq!(significand.to_bits(), x.to_bits()),
            FpCategory::Zero => {
                assert_eq!(significand.to_bits(), x.to_bits());
                assert_eq!(exponent, 0);
            }
            _ => {
                assert!((0.5..1.0).contains(&significand.abs()));
                assert_eq!(metal::ldexp(significand, exponent).to_bits(), x.to_bits());
            }
        }
    });
}

#[test]
fn test_hypot_usual() {
    (0..=0xFFFF).for_each(|i| {
        let x = f32::from_bits(0x10001 * i);

        (0..=0xFFFF).for_each(|j| {
            let y = f32::from_bits(j << 16);
            assert!(is(metal::hypot(x, y), core_math::hypotf(x, y)));
        });
    });
}

#[test]
fn test_hypot_worst_cases() {
    include_bytes!("hypot.wc")
        .chunks_exact(8)
        .for_each(|chunk| {
            let x = f32::from_le_bytes(chunk[..4].try_into().expect("4 bytes"));
            let y = f32::from_le_bytes(chunk[4..].try_into().expect("4 bytes"));
            assert!(is(metal::hypot(x, y), core_math::hypotf(x, y)));
        });
}
