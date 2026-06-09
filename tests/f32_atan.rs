mod common;

#[test]
fn test_atan() {
    common::test_all_f32(metallic::atanf, core_math::atanf);
}
