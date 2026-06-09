mod common;

#[test]
fn test_asinh() {
    common::test_all_f32(metallic::asinhf, core_math::asinhf);
}
