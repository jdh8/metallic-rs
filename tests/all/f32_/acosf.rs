use crate::common;

#[test]
fn test_acos() {
    common::test_all_f32(metallic::acosf, core_math::acosf);
}
