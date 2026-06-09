mod common;

#[test]
fn test_acosh() {
    common::test_all_f32(metallic::acoshf, core_math::acoshf);
}
