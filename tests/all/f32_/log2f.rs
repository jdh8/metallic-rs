use crate::common;

#[test]
fn test_log2() {
    common::test_all_f32(metallic::log2f, core_math::log2f);
}
