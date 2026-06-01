mod common;
use metallic::f32 as metal;

#[test]
fn test_lgamma() {
    common::test_all_f32(metal::lgamma, core_math::lgammaf);
}
