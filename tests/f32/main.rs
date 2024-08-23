mod huge;
mod ldexp;
use core::num::FpCategory;
use metallic::f32 as metal;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x87;

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
            let f = f(x);
            let g = g(x);

            (!is(f, g)).then(|| Some(println!("{x:e}: {f:e} != {g:e}")))
        })
        .count();

    assert!(count == 0, "There are {count} mismatches");
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

/// Check if `result` is within the nearby `f32` representations of `expected`
///
/// Due to [the Table Maker's Dilemma][dilemma], it is infeasible to implement a
/// correctly-rounded (error < 0.5 ulp) transcendental function.  However,
/// faithful rounding (error < 1 ulp) is usually achievable.
///
/// [dilemma]: https://hal-lara.archives-ouvertes.fr/hal-02101765/document
///
/// If `expected` has an exact `f32` representation, `result` must be that
/// value.  Otherwise, `expected` has two `f32` neighbors, and `result` must be
/// either of them.
fn is_faithful_rounding(result: f32, expected: f64) -> bool {
    #[allow(clippy::cast_possible_truncation)]
    if is(result, expected as f32) {
        return true;
    }

    let next_up = f64::from(metal::next_up(result));
    let next_down = f64::from(metal::next_down(result));
    next_down < expected && expected < next_up
}

// Code repetition is intentional for future removal of this function
fn test_bivariate_faithful(f: impl Fn(f32, f32) -> f32, g: impl Fn(f64, f64) -> f64) {
    let count = (0..=u32::MAX)
        .filter_map(|bits| {
            let x = f32::from_bits(0x10001 * (bits >> 16));
            let y = f32::from_bits(bits << 16);
            let f = f(x, y);
            let g = g(x.into(), y.into());

            (!is_faithful_rounding(f, g)).then(|| Some(println!("{x:e}, {y:e}: {f:e} != {g:e}")))
        })
        .count();

    assert!(count == 0, "There are {count} mismatches");
}

fn test_bivariate_correct(f: impl Fn(f32, f32) -> f32, g: impl Fn(f32, f32) -> f32) {
    let count = (0..=u32::MAX)
        .filter_map(|bits| {
            let x = f32::from_bits(0x10001 * (bits >> 16));
            let y = f32::from_bits(bits << 16);
            let f = f(x, y);
            let g = g(x, y);

            (!is(f, g)).then(|| Some(println!("{x:e}, {y:e}: {f:e} != {g:e}")))
        })
        .count();

    assert!(count == 0, "There are {count} mismatches");
}

#[test]
fn test_hypot() {
    test_bivariate_correct(metal::hypot, core_math::hypotf);
}

#[test]
fn test_powf() {
    test_bivariate_faithful(metal::powf, core_math::pow);
}
