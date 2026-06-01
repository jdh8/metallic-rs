mod common;
use metallic::f32 as metal;

#[test]
fn test_atan() {
    common::test_all_f32(metal::atan, core_math::atanf);
}
