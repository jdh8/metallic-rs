mod common;
use metallic::f32 as metal;

#[test]
fn test_acos() {
    common::test_all_f32(metal::acos, core_math::acosf);
}
