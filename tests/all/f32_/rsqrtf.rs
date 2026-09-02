use crate::common;

/// Exhaustive bit-exact comparison against the core-math oracle over all 2³²
/// inputs — a complete correct-rounding certification.
#[test]
fn test_rsqrtf() {
    common::test_all_f32(metallic::rsqrtf, core_math::rsqrtf);
}
