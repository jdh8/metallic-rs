mod common;

/// Exhaustive bit-exact comparison against the core-math oracle over all 2³²
/// inputs — a complete correct-rounding certification.
#[test]
fn test_exp10m1f() {
    common::test_all_f32(metallic::exp10m1f, core_math::exp10m1f);
}
