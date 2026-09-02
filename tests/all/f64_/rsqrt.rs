use crate::common;

#[test]
fn test_parser() {
    assert!(common::parse_case_file("rsqrt.wc", common::parse_f64).count() == 9922);
}

#[test]
fn test_rsqrt() {
    common::test_univariate_cases(
        metallic::rsqrt,
        core_math::rsqrt,
        common::parse_case_file("rsqrt.wc", common::parse_f64)
            .chain((0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits)),
    );
}

/// Dense sweep of the fast leg over `[1, 4)` — one full period of the exponent
/// parity — bit-exact vs the core-math oracle.
#[test]
fn test_rsqrt_dense_band() {
    let lb = 1.0_f64.to_bits();
    let hb = 4.0_f64.to_bits();
    let band = (lb..hb)
        .step_by(((hb - lb) / 2_000_000) as usize)
        .map(f64::from_bits);
    common::test_univariate_cases(metallic::rsqrt, core_math::rsqrt, band);
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_rsqrt_worst_cases() {
    common::test_worst_univariate("rsqrt", metallic::rsqrt, core_math::rsqrt);
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_rsqrt_worst_faithful() {
    common::test_worst_faithful("rsqrt", metallic::rsqrt, core_math::rsqrt, 1);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.  Representation-uniform over all
/// `f64`, so subnormals, the huge-`x` seed branch, and the sign/NaN specials
/// all appear.
#[cfg(feature = "mpfr")]
#[test]
fn test_rsqrt_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).recip_sqrt().to_f64();
    common::mpfr_sweep_univariate(
        metallic::rsqrt,
        cr,
        |i| f64::from_bits(common::mix64(i)),
        2_000_000,
    );
}
