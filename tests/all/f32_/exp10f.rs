use crate::common;

#[test]
fn test_exp10() {
    common::test_all_f32(metallic::exp10f, core_math::exp10f);
}
