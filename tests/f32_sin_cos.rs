mod common;
use metallic::f32 as metal;

#[test]
fn test_sin_cos() {
    common::test_all_f32(metal::sin_cos, core_math::sincosf);
}
