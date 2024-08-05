use core::f64;
use metallic::f32 as lib;

fn is_faithful_rounding(result: f32, expected: f64) -> bool {
    if expected.is_nan() {
        return result.is_nan();
    }

    let next_up = f64::from(lib::next_up(result));
    let next_down = f64::from(lib::next_down(result));
    next_down <= expected && expected <= next_up
}

#[test]
fn test_exp() {
    (0..u32::MAX).step_by(777).for_each(|i| {
        let x = f32::from_bits(i);
        assert!(is_faithful_rounding(lib::exp(x), f64::from(x).exp()));
    });
}
