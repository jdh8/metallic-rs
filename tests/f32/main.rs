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

    (0..u32::MAX).for_each(|i| {
        let x = f32::from_bits(i);
        assert!(is_faithful_rounding(ours(x), std_f64(f64::from(x))));
    });
}

macro_rules! test_unary {
    ($name:ident) => {
        #[test]
        fn $name() {
            test_unary(metal::$name, f32::$name, f64::$name);
        }
    };
}

test_unary!(cbrt);
test_unary!(exp);
test_unary!(exp2);
test_unary!(exp_m1);

#[test]
fn exp10() {
    assert!(metal::exp10(1.0).eq(&1e1));
    assert!(metal::exp10(2.0).eq(&1e2));
    assert!(metal::exp10(3.0).eq(&1e3));
    assert!(metal::exp10(4.0).eq(&1e4));
    assert!(metal::exp10(5.0).eq(&1e5));
    assert!(metal::exp10(6.0).eq(&1e6));
    assert!(metal::exp10(7.0).eq(&1e7));
    assert!(metal::exp10(8.0).eq(&1e8));
    assert!(metal::exp10(9.0).eq(&1e9));
    assert!(metal::exp10(10.0).eq(&1e10));

    assert!(metal::exp10(-1.0).eq(&1e-1));
    assert!(metal::exp10(-2.0).eq(&1e-2));
    assert!(metal::exp10(-3.0).eq(&1e-3));
    assert!(metal::exp10(-4.0).eq(&1e-4));
    assert!(metal::exp10(-5.0).eq(&1e-5));
    assert!(metal::exp10(-6.0).eq(&1e-6));
    assert!(metal::exp10(-7.0).eq(&1e-7));
    assert!(metal::exp10(-8.0).eq(&1e-8));
    assert!(metal::exp10(-9.0).eq(&1e-9));
    assert!(metal::exp10(-10.0).eq(&1e-10));

    test_unary(metal::exp10, libm::exp10f, libm::exp10);
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
