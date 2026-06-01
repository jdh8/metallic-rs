mod common;
use metallic::f32 as metal;

#[test]
fn test_cbrt() {
    common::test_all_f32(metal::cbrt, core_math::cbrtf);
}
