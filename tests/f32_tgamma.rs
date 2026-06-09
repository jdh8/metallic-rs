mod common;

#[test]
fn test_tgamma() {
    common::test_all_f32(metallic::tgammaf, core_math::tgammaf);
}
