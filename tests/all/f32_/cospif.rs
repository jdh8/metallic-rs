use crate::common;

/// Exhaustive bit-exact comparison against the core-math oracle over all 2³²
/// inputs — a complete correct-rounding certification.
#[test]
fn test_cospif() {
    common::test_all_f32(metallic::cospif, core_math::cospif);
}
