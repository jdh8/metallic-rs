mod ldexp;
use core::num::FpCategory;
use metallic::f32 as metal;

fn is_faithful_rounding(result: f32, expected: f64) -> bool {
    if expected.is_nan() {
        return result.is_nan();
    }

    let next_up = f64::from(metal::next_up(result));
    let next_down = f64::from(metal::next_down(result));
    (next_down..=next_up).contains(&expected)
}

fn test_unary(
    ours: impl Fn(f32) -> f32,
    std_f32: impl Fn(f32) -> f32,
    std_f64: impl Fn(f64) -> f64,
    step: usize,
) {
    assert_eq!(ours(0.0).to_bits(), std_f32(0.0).to_bits());
    assert_eq!(ours(-0.0).to_bits(), std_f32(-0.0).to_bits());

    assert_eq!(
        ours(f32::INFINITY).to_bits(),
        std_f32(f32::INFINITY).to_bits(),
    );

    assert_eq!(
        ours(f32::NEG_INFINITY).to_bits(),
        std_f32(f32::NEG_INFINITY).to_bits(),
    );

    (0..u32::MAX).step_by(step).for_each(|i| {
        let x = f32::from_bits(i);
        assert!(is_faithful_rounding(ours(x), std_f64(f64::from(x))));
    });
}

macro_rules! test_unary {
    ($name:ident, $step:expr) => {
        #[test]
        fn $name() {
            test_unary(metal::$name, f32::$name, f64::$name, $step);
        }
    };
}

test_unary!(cbrt, 69);
test_unary!(exp, 69);
test_unary!(exp_m1, 69);

#[test]
fn frexp() {
    (0..u32::MAX).step_by(69).for_each(|i| {
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
