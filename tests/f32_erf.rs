mod common;

#[test]
fn test_erf() {
    common::test_all_f32(metallic::erff, core_math::erff);
}
