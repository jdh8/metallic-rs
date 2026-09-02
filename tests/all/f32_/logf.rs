use crate::common;

#[test]
fn test_ln() {
    common::test_all_f32(metallic::logf, core_math::logf);
}
