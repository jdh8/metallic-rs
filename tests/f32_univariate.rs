mod common;

use common::*;
use core::num::FpCategory;
use metallic::f32 as metal;

/// Check if `f` returns the same result as `g` for every `f32` values
///
/// By "same result", I mean semantic identity as defined by [`is`].
fn test_identity<Output: Identity + core::fmt::Debug>(
    f: impl Fn(f32) -> Output,
    g: impl Fn(f32) -> Output,
) {
    test_univariate_cases(f, g, (0..=u32::MAX).map(f32::from_bits));
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
fn test_cosh() {
    test_identity(metal::cosh, core_math::coshf);
}

#[test]
fn test_sinh() {
    test_identity(metal::sinh, core_math::sinhf);
}

#[test]
fn test_tanh() {
    test_identity(metal::tanh, core_math::tanhf);
}

#[test]
fn test_acos() {
    test_identity(metal::acos, core_math::acosf);
}

#[test]
fn test_asin() {
    test_identity(metal::asin, core_math::asinf);
}

#[test]
fn test_atan() {
    test_identity(metal::atan, core_math::atanf);
}

#[test]
fn test_cos() {
    test_identity(metal::cos, core_math::cosf);
}

#[test]
fn test_sin() {
    test_identity(metal::sin, core_math::sinf);
}

#[test]
fn test_sin_cos() {
    test_identity(metal::sin_cos, core_math::sincosf);
}

#[test]
fn test_tan() {
    test_identity(metal::tan, core_math::tanf);
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
fn test_hypot() {
    test_bivariate_correct(metal::hypot, core_math::hypotf);
}

#[test]
fn test_powf() {
    test_bivariate_faithful(metal::powf, core_math::pow);
}
