mod common;

#[test]
fn test_log10p1f() {
    common::test_all_f32(metallic::log10p1f, core_math::log10p1f);
}
