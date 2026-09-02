use crate::common;

#[test]
fn test_lgamma() {
    common::test_all_f32(metallic::lgammaf, core_math::lgammaf);
}
