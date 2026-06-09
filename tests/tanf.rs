mod common;

#[test]
fn test_tan() {
    common::test_all_f32(metallic::tanf, core_math::tanf);
}
