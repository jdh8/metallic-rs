use crate::common;

#[test]
fn test_erfc() {
    common::test_all_f32(metallic::erfcf, core_math::erfcf);
}
