use crate::common;

#[test]
fn test_round() {
    common::test_all_f32(metallic::roundf, f32::round);
}
