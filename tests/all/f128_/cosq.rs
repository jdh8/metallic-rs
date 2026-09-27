use crate::common;
use crate::common128;

use common::Identity as _;

// CORE-MATH's `cosq` is the oracle: the sweeps and its own hard-to-round
// corpus (`tests/cases/cosq.wc`) gate bit-exact against it.  Beside that
// corpus, `f128_cos.wc` keeps metallic's own, which carries its MPFR answers
// (`examples/gen_f128_trig_cases.rs`): edges, the per-binade convergents of
// π/2, and a near-midpoint scan.  Upstream's `cosq.c` and metallic's share a
// structure, so the checks that do not go through it stay: the f64 oracle
// covers every magnitude up to 2^1024 with no MPFR, and the MPFR sweeps are
// the independent cross-check.

#[test]
fn test_parser() {
    assert_eq!(
        common::parse_case_file("cosq.wc", common128::parse_f128).count(),
        98_961
    );
}

/// Size of `tests/cases/f128_cos.wc` (kept in sync with the generator).
const CORPUS_LEN: usize = 96_265;

const SAMPLE_COUNT: u64 = 200_000;

/// A signed value with the unbiased exponent drawn from `range`.
fn banded(i: u64, range: core::ops::RangeInclusive<i32>) -> f128 {
    let bits = common128::mix128(i);
    let span = (range.end() - range.start() + 1) as u128;
    let exponent = (*range.start() + 16383) as u128 + (bits >> 112 & 0x7fff) % span;
    f128::from_bits((bits & (1 << 127)) | exponent << 112 | (bits & (1 << 112) - 1))
}

/// The direct band `[2^-57, 2^-8)`, where the argument is its own reduced angle.
fn direct(i: u64) -> f128 {
    banded(i, -57..=-9)
}

/// The reduction band up to 2^20, where the breakpoints and both legs work.
fn reduced(i: u64) -> f128 {
    banded(i, -8..=20)
}

/// Representation-uniform bits: every binade, subnormals, and the specials.
fn wide(i: u64) -> f128 {
    f128::from_bits(common128::mix128(i))
}

#[test]
fn test_cosq_corpus() {
    let cases: Vec<[f128; 2]> =
        common::parse_case_file("f128_cos.wc", common128::parse_f128_pair).collect();
    assert_eq!(
        cases.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );

    common::truncate_errors(cases.into_iter().filter_map(|[x, want]| {
        let got = metallic::cosq(x);
        (!got.is(&want)).then(|| println!("cosq({x:?}) = {got:?} != {want:?} (correct)"))
    }));
}

#[test]
fn test_cosq() {
    for sampler in [direct, reduced, wide] {
        common::test_univariate_cases(
            metallic::cosq,
            core_math::cosq,
            (0..SAMPLE_COUNT).map(sampler),
        );
    }
}

#[test]
fn test_cosq_worst_cases() {
    common128::test_worst_univariate_f128("cos", metallic::cosq, core_math::cosq);
}

/// Every `f64` is a binary128 value, and its correctly rounded `f64` cosine can
/// differ from the binary128 one rounded down to `f64` by at most one ulp
/// (the double rounding).  With CORE-MATH's `cos` as the oracle this checks
/// the reduction at every magnitude up to 2^1024 with no MPFR at all.
#[test]
fn test_cosq_vs_f64() {
    common::truncate_errors((0..SAMPLE_COUNT).filter_map(|i| {
        let h = common::mix64(i);
        let exponent = 1023 - 57 + (h >> 52 & 0x7ff) % 1081;
        let x = f64::from_bits((h & 1 << 63) | exponent << 52 | (h & (1 << 52) - 1));
        let got = metallic::cosq(x as f128) as f64;
        let want = core_math::cos(x);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("cosq({x:e}) = {got:e} != {want:e}"))
    }));
}

#[test]
fn test_cosq_even() {
    common::truncate_errors((0..SAMPLE_COUNT).filter_map(|i| {
        let x = wide(i);
        let (got, want) = (metallic::cosq(-x), metallic::cosq(x));
        (!got.is(&want)).then(|| println!("cosq({:?}) = {got:?} != {want:?}", -x))
    }));
}

#[test]
fn test_cosq_special() {
    assert!(metallic::cosq(0.0).is(&1.0));
    assert!(metallic::cosq(-0.0).is(&1.0));
    assert!(metallic::cosq(core::f128::consts::PI).is(&-1.0));
    assert!(metallic::cosq(-core::f128::consts::PI).is(&-1.0));
    assert!(metallic::cosq(f128::INFINITY).is_nan());
    assert!(metallic::cosq(f128::NEG_INFINITY).is_nan());
    assert!(metallic::cosq(f128::NAN).is_nan());
    // cos(x) rounds to 1 below the quadratic term's reach.
    assert!(metallic::cosq(f128::from_bits(1)).is(&1.0));
    assert!(metallic::cosq(-f128::MIN_POSITIVE).is(&1.0));
}

#[cfg(feature = "mpfr")]
fn oracle(x: f128) -> f128 {
    use rug::float::Round::Nearest;

    metallic::f128_mpfr::cr_unop(x, |y| y.cos_round(Nearest))
}

/// `round(k·π/2) + 2^-t` for `k < 2^30` and `t ∈ [7, 130]`: residuals from
/// the fast leg's whole range down past its hand-over to the accurate leg.
#[cfg(feature = "mpfr")]
fn near_multiple(i: u64) -> f128 {
    use rug::{Float, float::Constant, float::Round};

    let bits = common128::mix128(i);
    let k = (bits >> 64) % (1 << 30) + 1;
    let t = 7 + (bits % 124) as i64;
    let x = Float::with_val(200, Constant::Pi) * k as u32 / 2u32;
    let x = x.to_f128_round(Round::Nearest);
    let sign = if bits & 1 << 127 == 0 { 1.0 } else { -1.0 };

    sign * (x + f128::from_bits(((16383 - t) as u128) << 112))
}

#[cfg(feature = "mpfr")]
#[test]
fn test_cosq_vs_mpfr() {
    for sampler in [direct, reduced, wide, near_multiple] {
        common128::mpfr_sweep_univariate_f128(metallic::cosq, oracle, sampler, SAMPLE_COUNT);
    }
}
