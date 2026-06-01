mod common;
use metallic::f32 as metal;

#[test]
fn test_ln() {
    common::test_all_f32(metal::ln, core_math::logf);
}
