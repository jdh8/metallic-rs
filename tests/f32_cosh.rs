mod common;
use metallic::f32 as metal;

#[test]
fn test_cosh() {
    common::test_all_f32(metal::cosh, core_math::coshf);
}
