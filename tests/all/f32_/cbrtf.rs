use crate::common;

#[test]
fn test_cbrt() {
    common::test_all_f32(metallic::cbrtf, core_math::cbrtf);
}
