mod common;
use metallic::f32 as metal;

#[test]
fn test_log10() {
    common::test_all_f32(metal::log10, core_math::log10f);
}
