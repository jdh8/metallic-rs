mod common;
use metallic::f32 as metal;

#[test]
fn test_sin() {
    common::test_all_f32(metal::sin, core_math::sinf);
}
