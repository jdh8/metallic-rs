mod common;

#[test]
fn test_sin_cos() {
    common::test_all_f32(metallic::sincosf, core_math::sincosf);
}
