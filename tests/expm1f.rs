mod common;

#[test]
fn test_exp_m1() {
    common::test_all_f32(metallic::expm1f, core_math::expm1f);
}
