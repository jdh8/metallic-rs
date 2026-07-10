mod common;

#[test]
fn test_log2p1f() {
    common::test_all_f32(metallic::log2p1f, core_math::log2p1f);
}
