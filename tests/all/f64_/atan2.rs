use crate::common;

#[test]
fn test_atan2() {
    // A dense (y, x) grid over [-5, 5]² plus hashed wide-magnitude pairs and the
    // ∞/0/sign special cases.
    let grid = (0..2500u64).flat_map(|i| {
        (0..2500u64).map(move |j| {
            let y = metallic::fma(f64::from(i as u32), 10.0 / 2500.0, -5.0);
            let x = metallic::fma(f64::from(j as u32), 10.0 / 2500.0, -5.0);
            [y, x]
        })
    });
    let wide = (0..2_000_000u64).map(|i| {
        let y = f64::from_bits(i.wrapping_mul(0x2545_F491_4F6C_DD1D));
        let x = f64::from_bits(i.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xDEAD);
        [y, x]
    });
    common::test_bivariate_cases(metallic::atan2, core_math::atan2, grid.chain(wide));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_atan2_worst_cases() {
    common::test_worst_bivariate("atan2", metallic::atan2, core_math::atan2);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_atan2_worst_faithful() {
    common::test_worst_faithful_bivariate("atan2", metallic::atan2, core_math::atan2, 1);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_atan2_vs_mpfr() {
    let cr = |y: f64, x: f64| {
        rug::Float::with_val(200, y)
            .atan2(&rug::Float::with_val(200, x))
            .to_f64()
    };
    common::mpfr_sweep_bivariate(
        metallic::atan2,
        cr,
        // Independent value-uniform `(y, x)` over a wide magnitude band and all
        // four sign quadrants, so the sweep exercises every quadrant fold and the
        // `|y| ≷ |x|` swap.
        |i| {
            let y = common::uniform(common::mix64(i), -1.0, 1.0)
                * (2.0f64).powi((common::mix64(i ^ 0x1234_5678) % 200) as i32 - 100);
            let x = common::uniform(common::mix64(i ^ 0x9e37_79b9), -1.0, 1.0)
                * (2.0f64).powi((common::mix64(i ^ 0xdead_beef) % 200) as i32 - 100);
            [y, x]
        },
        2_000_000,
    );
}

/// Issue #11: a power-of-two `y` against `x = 1 − 2⁻⁵³` puts the exact ratio
/// `2⁻¹⁰⁶` past a rounding midpoint, where atan's `−q³/3` decides the last
/// bit — the double-double tier must not return the bare IEEE quotient.  The
/// whole exact-scaling family, both signs, every quadrant, and a neighbour scan
/// around `x = 1`, `y = 2⁻ᵏ` for `k = 20..=70`.
#[test]
fn test_atan2_issue_11() {
    let one = f64::from_bits(1.0f64.to_bits() - 1);
    let quadrants = |y: f64, x: f64| {
        [
            [y, x],
            [-y, x],
            [y, -x],
            [-y, -x],
            [x, y],
            [-x, y],
            [x, -y],
            [-x, -y],
        ]
    };
    let scaled = (-970..=1024_i64).flat_map(move |e| {
        // `(1 − 2⁻⁵³)·2ᵉ`: biased exponent `1022 + e`, all-ones significand.
        let x = f64::from_bits((((1022 + e) as u64) << 52) | (u64::MAX >> 12));
        let y = f64::from_bits(((1023 + e - 52) as u64) << 52);
        quadrants(y, x)
    });
    let neighbours = (20..=70_i64).flat_map(move |k| {
        (-4..=4_i64).flat_map(move |dx| {
            (-4..=4_i64).flat_map(move |dy| {
                let x = f64::from_bits(one.to_bits().wrapping_add_signed(dx));
                let y = f64::from_bits((((1023 - k) as u64) << 52).wrapping_add_signed(dy));
                quadrants(y, x)
            })
        })
    });
    common::test_bivariate_cases(metallic::atan2, core_math::atan2, scaled.chain(neighbours));
}
