mod common;
use metallic::f32 as metal;

#[test]
fn test_exp10() {
    common::test_all_f32(metal::exp10, core_math::exp10f);
}
