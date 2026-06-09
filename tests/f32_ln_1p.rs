mod common;

#[test]
fn test_ln_1p() {
    common::test_all_f32(metallic::log1pf, core_math::log1pf);
}
