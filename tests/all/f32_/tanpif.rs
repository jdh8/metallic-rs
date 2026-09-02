use crate::common;

/// Exhaustive bit-exact comparison against the core-math oracle over all 2³²
/// inputs — a complete correct-rounding certification.
#[test]
fn test_tanpif() {
    common::test_all_f32(metallic::tanpif, core_math::tanpif);
}
