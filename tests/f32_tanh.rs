mod common;

#[test]
fn test_tanh() {
    common::test_all_f32(metallic::tanhf, core_math::tanhf);
}
