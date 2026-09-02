use crate::common;

#[test]
fn test_exp2() {
    common::test_all_f32(metallic::exp2f, core_math::exp2f);
}
