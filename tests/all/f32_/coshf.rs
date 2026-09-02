use crate::common;

#[test]
fn test_cosh() {
    common::test_all_f32(metallic::coshf, core_math::coshf);
}
