mod common;

/// Map a 64-bit hash to a value-uniform `f64` in `[-half, half]`.
fn signed(hash: u64, half: f64) -> f64 {
    metallic::fma((hash >> 11) as f64 / (1u64 << 53) as f64, 2.0 * half, -half)
}

fn cases() -> impl Iterator<Item = [f64; 2]> {
    // A: x ∈ [1, 2) with hashed mantissa, y value-uniform in [−1100, 1100].  This
    // is the hard regime: log₂x is small, so the `×y` amplification of its error
    // is largest and the result lands densely near rounding boundaries.
    let near1 = (0..4_000_000u64).map(|i| {
        let xb = 0x3FF0_0000_0000_0000 | (i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 12);
        let y = signed(i.wrapping_mul(0xC2B2_AE3D_27D4_EB4F), 1100.0);
        [f64::from_bits(xb), y]
    });

    // B: wider positive x (any exponent), y value-uniform in [−40, 40].
    let wide = (0..2_000_000u64).map(|i| {
        let xb = (i.wrapping_mul(0x2545_F491_4F6C_DD1D) & 0x7FFF_FFFF_FFFF_FFFF) | 1;
        let y = signed(i.wrapping_mul(0x9E37_79B9_7F4A_7C15), 40.0);
        [f64::from_bits(xb), y]
    });

    // C: integer exponents on x ∈ [1, 2) and its negation (exercises the
    // negative-base / odd-even-exponent sign logic).
    let intexp = (0..1_000_000u64).flat_map(|i| {
        let xb = 0x3FF0_0000_0000_0000 | (i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 12);
        let x = f64::from_bits(xb);
        let n = (i % 600) as f64 - 300.0;
        [[x, n], [-x, n]]
    });

    // D: representation-uniform pairs — ∞, NaN, ±0, negative bases, subnormals.
    let repr = (0..2_000_000u64).map(|i| {
        let a = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
        let b = i.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xABCD_1234_5678_9876;
        [f64::from_bits(a), f64::from_bits(b)]
    });

    let edges = [
        [2.0, 0.5],
        [2.0, 3.0],
        [10.0, 3.0],
        [3.0, 100.0],
        [0.0, 2.0],
        [0.0, -1.0],
        [-2.0, 3.0],
        [-2.0, 2.0],
        [-2.0, 0.5],
        [1.0, f64::INFINITY],
        [f64::INFINITY, 2.0],
        [f64::NAN, 0.0],
        [1.0, f64::NAN],
        [f64::MAX, 2.0],
        [f64::MIN_POSITIVE, 0.5],
    ];

    near1.chain(wide).chain(intexp).chain(repr).chain(edges)
}

#[test]
fn test_powf() {
    common::test_bivariate_cases(metallic::pow, core_math::pow, cases());
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus: bit-exact vs the
/// `core-math` oracle.  This is the strict gate (issue #6).
#[test]
fn test_pow_worst_cases() {
    common::test_worst_bivariate("pow", metallic::pow, core_math::pow);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_pow_worst_faithful() {
    common::test_worst_faithful_bivariate("pow", metallic::pow, core_math::pow, 1);
}

/// Independent gold-standard cross-check against MPFR, guarding against a shared
/// CORE-MATH bug.  `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_pow_vs_mpfr() {
    use rug::Float;
    use rug::ops::Pow;
    // `x^y` at 250-bit precision, then rounded to f64 — the correctly-rounded
    // reference.  Matches `pow`'s domain conventions (rug returns NaN for a
    // negative base with a non-integer exponent, as `pow` does).
    let cr = |x: f64, y: f64| {
        Float::with_val(250, x)
            .pow(Float::with_val(250, y))
            .to_f64()
    };

    // Hard regime: x ∈ [1, 2) (small log₂x, max `×y` amplification) with large
    // |y|, plus a wide regime with any positive x and moderate y, and a strand of
    // negative bases with integer exponents (the sign-fold path).
    common::mpfr_sweep_bivariate(
        metallic::pow,
        cr,
        |i| {
            let h = common::mix64(i);
            match i % 3 {
                0 => {
                    let xb = 0x3FF0_0000_0000_0000 | (h >> 12);
                    let y = common::uniform(common::mix64(i ^ 0xABCD), -1100.0, 1100.0);
                    [f64::from_bits(xb), y]
                }
                1 => {
                    let xb = (h & 0x7FFF_FFFF_FFFF_FFFF) | 1;
                    let y = common::uniform(common::mix64(i ^ 0x1234), -40.0, 40.0);
                    [f64::from_bits(xb), y]
                }
                _ => {
                    let xb = 0x3FF0_0000_0000_0000 | (h >> 12);
                    let n = (common::mix64(i ^ 0x9999) % 600) as f64 - 300.0;
                    [-f64::from_bits(xb), n]
                }
            }
        },
        5_000_000,
    );
}
