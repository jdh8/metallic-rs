mod common;
use metallic::f32 as metal;

#[test]
fn test_erfc() {
    common::test_all_f32(metal::erfc, core_math::erfcf);
}
