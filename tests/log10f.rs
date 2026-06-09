mod common;

#[test]
fn test_log10() {
    common::test_all_f32(metallic::log10f, core_math::log10f);
}
