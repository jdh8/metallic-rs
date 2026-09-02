use crate::common;

#[test]
fn test_atanh() {
    common::test_all_f32(metallic::atanhf, core_math::atanhf);
}
