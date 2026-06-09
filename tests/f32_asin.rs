mod common;

#[test]
fn test_asin() {
    common::test_all_f32(metallic::asinf, core_math::asinf);
}
