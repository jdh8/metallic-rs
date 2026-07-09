mod common;

#[test]
fn test_parser() {
    let count = common::parse_case_file("sinpi.wc", common::parse_f64).count();
    assert!(count == 56_427, "parsed {count} cases");
}

#[test]
fn test_sinpi() {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    // Near-integer inputs exercise the relatively-exact zero regime and the
    // parity sign, ±half-integers the exact ±1 results.
    let near_grid = (0u64..=1_000_000).flat_map(|i| {
        #[allow(clippy::cast_possible_truncation)]
        let k = f64::from((i % 200) as u32) - 100.0;
        let delta = f64::from_bits(0x3C90_0000_0000_0000 + i); // ≈2⁻⁵⁴ up
        [k + delta, k - delta, k + 0.5 + delta, k + 0.5 - delta]
    });
    common::test_univariate_cases(
        metallic::sinpi,
        core_math::sinpi,
        common::parse_case_file("sinpi.wc", common::parse_f64)
            .chain(bits)
            .chain(near_grid),
    );
}

/// Dense sweep of the reduced band `(0, ½]` and its negation — the lean table
/// leg — bit-exact vs the core-math oracle.
#[test]
fn test_sinpi_dense_band() {
    let lb = f64::MIN_POSITIVE.to_bits();
    let hb = 0.5_f64.to_bits();
    let band = (lb..=hb)
        .step_by(((hb - lb) / 2_000_000) as usize)
        .map(f64::from_bits);
    let cases = band.clone().chain(band.map(|x| -x));
    common::test_univariate_cases(metallic::sinpi, core_math::sinpi, cases);
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_sinpi_worst_cases() {
    common::test_worst_univariate("sinpi", metallic::sinpi, core_math::sinpi);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_sinpi_worst_faithful() {
    common::test_worst_faithful("sinpi", metallic::sinpi, core_math::sinpi, 1);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.  Half the samples land
/// representation-uniform in `±(0, ½]` (the reduced band, tiny inputs
/// included), half over the full domain.
#[cfg(feature = "mpfr")]
#[test]
fn test_sinpi_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).sin_pi().to_f64();
    let sampler = |i: u64| {
        let h = common::mix64(i);
        let neg = h >> 63 != 0;
        let m = h & 0x7FFF_FFFF_FFFF_FFFF;
        let bits = if i & 1 == 0 {
            m % 0x3FE0_0000_0000_0001 // ±(0, ½]
        } else {
            m // full domain, incl. huge, ∞, NaN
        };
        let x = f64::from_bits(bits);
        if neg { -x } else { x }
    };
    common::mpfr_sweep_univariate(metallic::sinpi, cr, sampler, 2_000_000);
}
