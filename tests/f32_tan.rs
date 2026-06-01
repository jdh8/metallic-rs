mod common;
use metallic::f32 as metal;

#[test]
fn test_tan() {
    common::test_all_f32(metal::tan, core_math::tanf);
}
