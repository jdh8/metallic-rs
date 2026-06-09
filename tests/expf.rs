mod common;

#[test]
fn test_exp() {
    common::test_all_f32(metallic::expf, core_math::expf);
}
