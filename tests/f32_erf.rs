mod common;
use metallic::f32 as metal;

#[test]
fn test_erf() {
    common::test_all_f32(metal::erf, core_math::erff);
}
