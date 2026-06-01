mod common;
use metallic::f32 as metal;

#[test]
fn test_tgamma() {
    common::test_all_f32(metal::tgamma, core_math::tgammaf);
}
