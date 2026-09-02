use crate::common;

#[test]
fn test_cos() {
    common::test_all_f32(metallic::cosf, core_math::cosf);
}
