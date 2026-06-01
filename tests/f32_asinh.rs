mod common;
use metallic::f32 as metal;

#[test]
fn test_asinh() {
    common::test_all_f32(metal::asinh, core_math::asinhf);
}
