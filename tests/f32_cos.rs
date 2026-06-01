mod common;
use metallic::f32 as metal;

#[test]
fn test_cos() {
    common::test_all_f32(metal::cos, core_math::cosf);
}
