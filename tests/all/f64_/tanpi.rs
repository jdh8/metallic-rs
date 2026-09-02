use crate::common;

#[test]
fn test_parser() {
    let count = common::parse_case_file("tanpi.wc", common::parse_f64).count();
    assert!(count == 57_049, "parsed {count} cases");
}

#[test]
fn test_tanpi() {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    // Near-integer, near-pole (half-integer), and near-±¼ inputs exercise the
    // zeros, the cotangent cell, and the exact ±1 returns.
    let near_grid = (0u64..=1_000_000).flat_map(|i| {
        #[allow(clippy::cast_possible_truncation)]
        let k = f64::from((i % 200) as u32) - 100.0;
        let delta = f64::from_bits(0x3C90_0000_0000_0000 + i); // ≈2⁻⁵⁴ up
        [
            k + delta,
            k - delta,
            k + 0.5 + delta,
            k + 0.5 - delta,
            k + 0.25 + delta,
            k + 0.75 - delta,
        ]
    });
    common::test_univariate_cases(
        metallic::tanpi,
        core_math::tanpi,
        common::parse_case_file("tanpi.wc", common::parse_f64)
            .chain(bits)
            .chain(near_grid),
    );
}

/// Dense sweep across the small band, the grid leg, and a few periods.
#[test]
fn test_tanpi_dense_band() {
    let lb = f64::MIN_POSITIVE.to_bits();
    let hb = 4.0_f64.to_bits();
    let band = (lb..=hb)
        .step_by(((hb - lb) / 4_000_000) as usize)
        .map(f64::from_bits);
    let cases = band.clone().chain(band.map(|x| -x));
    common::test_univariate_cases(metallic::tanpi, core_math::tanpi, cases);
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_tanpi_worst_cases() {
    common::test_worst_univariate("tanpi", metallic::tanpi, core_math::tanpi);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_tanpi_worst_faithful() {
    common::test_worst_faithful("tanpi", metallic::tanpi, core_math::tanpi, 1);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.  Half the samples land
/// representation-uniform in `±(0, 1]`, half over the full domain.
#[cfg(feature = "mpfr")]
#[test]
fn test_tanpi_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).tan_pi().to_f64();
    let sampler = |i: u64| {
        let h = common::mix64(i);
        let neg = h >> 63 != 0;
        let m = h & 0x7FFF_FFFF_FFFF_FFFF;
        let bits = if i & 1 == 0 {
            m % 0x3FF0_0000_0000_0001 // ±(0, 1]
        } else {
            m // full domain, incl. huge, ∞, NaN
        };
        let x = f64::from_bits(bits);
        if neg { -x } else { x }
    };
    common::mpfr_sweep_univariate(metallic::tanpi, cr, sampler, 2_000_000);
}
