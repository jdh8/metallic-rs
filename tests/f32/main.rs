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

#[test]
fn test_exp() {
    assert!(metal::exp(0.0).eq(&1.0));
    assert!(metal::exp(-0.0).eq(&1.0));
    assert!(metal::exp(f32::INFINITY).eq(&f32::INFINITY));
    assert!(metal::exp(f32::NEG_INFINITY).to_bits() == 0);

    (0..u32::MAX).step_by(69).for_each(|i| {
        let x = f32::from_bits(i);
        assert!(is_faithful_rounding(metal::exp(x), f64::from(x).exp()));
    });
}

#[test]
fn test_frexp() {
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
