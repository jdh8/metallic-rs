mod common;
use metallic::f32 as metal;

#[test]
fn test_log2() {
    common::test_all_f32(metal::log2, core_math::log2f);
}
