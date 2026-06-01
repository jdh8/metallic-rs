mod common;
use metallic::f32 as metal;

#[test]
fn test_atanh() {
    common::test_all_f32(metal::atanh, core_math::atanhf);
}
