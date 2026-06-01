mod common;
use metallic::f32 as metal;

#[test]
fn test_exp_m1() {
    common::test_all_f32(metal::exp_m1, core_math::expm1f);
}
