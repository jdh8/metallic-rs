mod common;

/// Test cases stressing the hard-rounding regime and the full magnitude range.
fn cases() -> impl Iterator<Item = [f64; 2]> {
    // Both legs in [1, 2) with hashed mantissas: `big² + small²` then ranges
    // densely over [1, 8) and frequently lands near an f64 rounding boundary,
    // which is where correct rounding is hardest.
    let similar = (0..6_000_000_u64).map(|i| {
        let a = 0x3FF0_0000_0000_0000 | (i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 12);
        let b = 0x3FF0_0000_0000_0000 | (i.wrapping_mul(0xC2B2_AE3D_27D4_EB4F) >> 12);
        [f64::from_bits(a), f64::from_bits(b)]
    });

    // Wide, representation-uniform pairs: exercise scaling, the overflow guard,
    // subnormal results, and the ∞/NaN/zero special cases.
    let wide = (0..3_000_000_u64).map(|i| {
        let a = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        let b = i.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x1234_5678_9ABC_DEF0;
        [f64::from_bits(a), f64::from_bits(b)]
    });

    // Explicit edge cases: overflow boundary, subnormals, and exact triples.
    let edges = [
        [f64::MAX, f64::MAX],
        [f64::MAX, 0.0],
        [f64::MAX, f64::MIN_POSITIVE],
        [f64::from_bits(1), f64::from_bits(1)],
        [f64::from_bits(3), f64::from_bits(4)],
        [3.0, 4.0],
        [5.0e-323, 7.0e-323],
        [1.0, 0.0],
        [f64::INFINITY, f64::NAN],
        [f64::NAN, 1.0],
    ];

    similar.chain(wide).chain(edges)
}

#[test]
fn test_hypot() {
    common::test_bivariate_cases(metallic::hypot, core_math::hypot, cases());
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_hypot_worst_cases() {
    common::test_worst_bivariate("hypot", metallic::hypot, core_math::hypot);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_hypot_worst_faithful() {
    common::test_worst_faithful_bivariate("hypot", metallic::hypot, core_math::hypot, 1);
}
