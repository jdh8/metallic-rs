mod common;
use core::num::FpCategory;

#[test]
fn test_frexp() {
    (0..=u64::MAX).step_by((1 << 37) - 1337).for_each(|i| {
        let x = f64::from_bits(i);
        let (significand, exponent) = metallic::frexp(x);

        match x.classify() {
            FpCategory::Nan => assert!(significand.is_nan()),
            FpCategory::Infinite => assert_eq!(significand.to_bits(), x.to_bits()),
            FpCategory::Zero => {
                assert_eq!(significand.to_bits(), x.to_bits());
                assert_eq!(exponent, 0);
            }
            _ => {
                assert!((0.5..1.0).contains(&significand.abs()), "frexp({x:e})");
                assert_eq!(
                    metallic::ldexp(significand, exponent).to_bits(),
                    x.to_bits()
                );
            }
        }
    });
}
