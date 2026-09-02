use crate::common;

#[test]
fn test_sin() {
    common::test_all_f32(metallic::sinf, core_math::sinf);
}
