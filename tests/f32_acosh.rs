mod common;
use metallic::f32 as metal;

#[test]
fn test_acosh() {
    common::test_all_f32(metal::acosh, core_math::acoshf);
}
