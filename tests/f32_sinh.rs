mod common;
use metallic::f32 as metal;

#[test]
fn test_sinh() {
    common::test_all_f32(metal::sinh, core_math::sinhf);
}
