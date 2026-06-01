mod common;
use metallic::f32 as metal;

#[test]
fn test_exp() {
    common::test_all_f32(metal::exp, core_math::expf);
}
