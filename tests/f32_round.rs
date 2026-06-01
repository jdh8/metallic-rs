mod common;
use metallic::f32 as metal;

#[test]
fn test_round() {
    common::test_all_f32(metal::round, f32::round);
}
