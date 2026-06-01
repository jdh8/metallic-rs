mod common;
use metallic::f32 as metal;

#[test]
fn test_asin() {
    common::test_all_f32(metal::asin, core_math::asinf);
}
