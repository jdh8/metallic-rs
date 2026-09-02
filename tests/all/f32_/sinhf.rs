use crate::common;

#[test]
fn test_sinh() {
    common::test_all_f32(metallic::sinhf, core_math::sinhf);
}
