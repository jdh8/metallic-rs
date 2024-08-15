mod ldexp;
use core::{f32, num::FpCategory};
use metallic::f32 as metal;

/// Potential zeros and poles
const SINGULARITIES: [f32; 4] = [0.0, -0.0, f32::INFINITY, f32::NEG_INFINITY];

/// Semantic identity like `Object.is` in JavaScript
///
/// This function works around comparison issues with NaNs and signed zeros.
/// To be specific, `is(f32::NAN, f32::NAN)` but not `is(0.0, -0.0)`.
fn is(x: f32, y: f32) -> bool {
    x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan())
}

/// Check if the functions return the same value for the given inputs
fn test_special_values(
    f: impl Fn(f32) -> f32,
    g: impl Fn(f32) -> f32,
    values: impl Iterator<Item = f32>,
) {
    values.for_each(|x| assert!(is(f(x), g(x))));
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

/// Check if `f` is a faithful rounding of `g` for all `f32` values
fn test_faithful_rounding(f: impl Fn(f32) -> f32, g: impl Fn(f64) -> f64) {
    (0..=u32::MAX).for_each(|i| {
        let x = f32::from_bits(i);
        assert!(is_faithful_rounding(f(x), g(f64::from(x))));
    });
}

/// Check if `f` returns the same result as `g` for every `f32` values
///
/// By "same result", I mean semantic identity as defined by [`is`].
fn test_correct_rounding(f: impl Fn(f32) -> f32, g: impl Fn(f32) -> f32) {
    (0..=u32::MAX).for_each(|i| {
        let x = f32::from_bits(i);
        assert!(is(f(x), g(x)));
    });
}

/// Test suite for unary functions
macro_rules! test_unary {
    ($name:ident, $values:expr) => {
        #[test]
        fn $name() {
            test_special_values(metal::$name, f32::$name, $values);
            test_faithful_rounding(metal::$name, f64::$name);
        }
    };
    ($name:ident) => {
        test_unary!($name, SINGULARITIES.into_iter());
    };
}

#[test]
fn test_round() {
    test_correct_rounding(metal::round, f32::round);
}

#[test]
fn test_cbrt() {
    test_correct_rounding(metal::cbrt, core_math::cbrtf);
}

test_unary!(exp);
test_unary!(exp2);
test_unary!(exp_m1);
test_unary!(ln, core::iter::once(1.0).chain(SINGULARITIES));
test_unary!(ln_1p, core::iter::once(-1.0).chain(SINGULARITIES));

test_unary!(
    log2,
    #[allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)]
    (f32::MIN_EXP - f32::MANTISSA_DIGITS as i32..f32::MAX_EXP)
        .map(|x| (x as f32).exp2())
        .chain(SINGULARITIES)
);

test_unary!(
    log10,
    (0..11).map(|x| 10.0_f32.powi(x)).chain(SINGULARITIES)
);

test_unary!(acosh, core::iter::once(1.0).chain(SINGULARITIES));
test_unary!(asinh);
test_unary!(atanh, [1.0, -1.0].into_iter().chain(SINGULARITIES));

#[test]
fn exp10() {
    test_special_values(
        metal::exp10,
        libm::exp10f,
        (-10i8..=10).map(f32::from).chain(SINGULARITIES),
    );
    test_faithful_rounding(metal::exp10, libm::exp10);
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

/// Test suit for binary functions
macro_rules! test_binary {
    ($name:ident) => {
        #[test]
        fn $name() {
            (0..=0xFFFF).for_each(|i| {
                let x = f32::from_bits(0x10001 * i);

                (0..=0xFFFF).for_each(|j| {
                    let y = f32::from_bits(j << 16);

                    assert!(is_faithful_rounding(
                        metal::$name(x, y),
                        f64::$name(x.into(), y.into()),
                    ));
                });
            });
        }
    };
}

#[test]
fn test_hypot() {
    // Worst cases taken from [CORE-MATH]/src/binary32/hypot/hypotf.wc
    const WORST_CASES: [[f32; 2]; 20] = [
        [3.824_151_8e-16, -1.006_103_66e-19],
        [1.181_016_1e-10, 4.048_443e-14],
        [2.989_237_4e29, -3.299_689_3e32],
        [-8.334_588e-39, 1.667_698e-38],
        [6.765_83e-39, -4.832_257e-39],
        [1.814_624_1e20, -2.611_292_4e23],
        [3.402_823_5e38, 8.307_674_5e34],
        [3.402_823_5e38, 1e-45],
        [6.710_95e7, 1.556_614_1e7],
        [6.710_957_6e7, 1.386_722_9e7],
        [6.710_965e7, 1.500_396_5e7],
        [6.710_985e7, 9.827_915e6],
        [6.710_989e7, 1.174_075_7e7],
        [6.711_023e7, 1.268_156_5e7],
        [6.711_025e7, 9.634_667e6],
        [6.711_069_6e7, 9.967_029e6],
        [6.711_069_6e7, 1.616_553_9e7],
        [6.711_071e7, 1.133_352_5e7],
        [6.711_075e7, 1.044_855_5e7],
        [1.342_221_3e8, 1.156_074_7e7],
    ];

    for [x, y] in WORST_CASES {
        assert!(is(metal::hypot(x, y), core_math::hypotf(x, y)));
    };

    (0..=0xFFFF).for_each(|i| {
        let x = f32::from_bits(0x10001 * i);

        (0..=0xFFFF).for_each(|j| {
            let y = f32::from_bits(j << 16);
            assert!(is(metal::hypot(x, y), core_math::hypotf(x, y)));
        });
    });
}

test_binary!(log);
test_binary!(powf);
