mod common;
use metallic::f32 as metal;

#[test]
fn test_exp2() {
    common::test_all_f32(metal::exp2, core_math::exp2f);
}
