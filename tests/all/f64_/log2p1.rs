use crate::common;

#[test]
fn test_log2p1() {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    let near_zero = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 1.5 / 2_000_000.0, -0.5));
    common::test_univariate_cases(metallic::log2p1, core_math::log2p1, bits.chain(near_zero));
}

/// Dense both-signs sweep of the mid-`|x|` polynomial leg's band `[2⁻⁸, 2⁻⁴)`
/// (the `LN1P_WIDE_ZIV_SCALE`-gated leg), bit-exact vs the core-math oracle.
#[test]
fn test_log2p1_wide_band() {
    let lb = 3.90625e-3_f64.to_bits();
    let hb = 0.0625_f64.to_bits();
    let band = (lb..hb)
        .step_by(((hb - lb) / 1_050_000) as usize)
        .map(f64::from_bits);
    let cases = band.clone().chain(band.map(|x| -x));
    common::test_univariate_cases(metallic::log2p1, core_math::log2p1, cases);
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_log2p1_worst_cases() {
    common::test_worst_univariate("log2p1", metallic::log2p1, core_math::log2p1);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_log2p1_worst_faithful() {
    common::test_worst_faithful("log2p1", metallic::log2p1, core_math::log2p1, 1);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.  Half the samples land
/// representation-uniform in the polynomial legs' bands `±(0, 2⁻⁴)`, half over
/// the full domain: `(−1, 0)` and the positive reals up to +∞/NaN.
#[cfg(feature = "mpfr")]
#[test]
fn test_log2p1_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).log2_1p().to_f64();
    let sampler = |i: u64| {
        let h = common::mix64(i);
        let neg = h >> 63 != 0;
        let m = h & 0x7FFF_FFFF_FFFF_FFFF;
        let bits = if i & 1 == 0 {
            m % 0x3FB0_0000_0000_0000 // the ±(0, 2⁻⁴) polynomial-leg bands
        } else if neg {
            m % 0x3FF0_0000_0000_0000 // (−1, 0)
        } else {
            m // full positive domain, incl. +∞ and NaN
        };
        let x = f64::from_bits(bits);
        if neg { -x } else { x }
    };
    common::mpfr_sweep_univariate(metallic::log2p1, cr, sampler, 2_000_000);
}

/// The exact cases `log2p1(2ᵏ − 1) = k` must come out exactly integer.
#[test]
fn test_log2p1_exact_cases() {
    for k in -53_i32..=53 {
        if k == 0 {
            continue;
        }
        let x = f64::powi(2.0, k) - 1.0; // exact: |2ᵏ − 1| needs ≤ 53 bits
        assert_eq!(metallic::log2p1(x), f64::from(k), "log2p1(2^{k} - 1)");
    }
}
