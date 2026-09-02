use crate::common;
use crate::common128;

use common::Identity as _;

// CORE-MATH has no `tanq` at all — not even a binary128 corpus — so the strict
// gate replays a home-grown corpus that carries its own MPFR answers
// (`examples/gen_f128_trig_cases.rs`) and runs under plain `--features f128`.
// The parser count keeps the corpus from rotting unnoticed, the f64 oracle
// covers every magnitude up to 2^1024 with no MPFR, and the MPFR sweeps stay
// the independent cross-check.

/// Size of `tests/cases/tanq.wc` (kept in sync with the generator).
const CORPUS_LEN: usize = 39_597;

const SAMPLE_COUNT: u64 = 200_000;

/// A signed value with the unbiased exponent drawn from `range`.
#[cfg(feature = "mpfr")]
fn banded(i: u64, range: core::ops::RangeInclusive<i32>) -> f128 {
    let bits = common128::mix128(i);
    let span = (range.end() - range.start() + 1) as u128;
    let exponent = (*range.start() + 16383) as u128 + (bits >> 112 & 0x7fff) % span;
    f128::from_bits((bits & (1 << 127)) | exponent << 112 | (bits & (1 << 112) - 1))
}

/// The direct band `[2^-57, 2^-8)`, where the argument is its own reduced angle.
#[cfg(feature = "mpfr")]
fn direct(i: u64) -> f128 {
    banded(i, -57..=-9)
}

/// The reduction band up to 2^20, where the breakpoints and both legs work.
#[cfg(feature = "mpfr")]
fn reduced(i: u64) -> f128 {
    banded(i, -8..=20)
}

/// Representation-uniform bits: every binade, subnormals, and the specials.
fn wide(i: u64) -> f128 {
    f128::from_bits(common128::mix128(i))
}

#[test]
fn test_tanq_corpus() {
    let cases: Vec<[f128; 2]> =
        common::parse_case_file("tanq.wc", common128::parse_f128_pair).collect();
    assert_eq!(
        cases.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );

    common::truncate_errors(cases.into_iter().filter_map(|[x, want]| {
        let got = metallic::tanq(x);
        (!got.is(&want)).then(|| println!("tanq({x:?}) = {got:?} != {want:?} (correct)"))
    }));
}

/// Every `f64` is a binary128 value, and its correctly rounded `f64` tangent
/// can differ from the binary128 one rounded down to `f64` by at most one ulp
/// (the double rounding).  With CORE-MATH's `tan` as the oracle this checks
/// the reduction at every magnitude up to 2^1024 with no MPFR at all.
#[test]
fn test_tanq_vs_f64() {
    common::truncate_errors((0..SAMPLE_COUNT).filter_map(|i| {
        let h = common::mix64(i);
        let exponent = 1023 - 57 + (h >> 52 & 0x7ff) % 1081;
        let x = f64::from_bits((h & 1 << 63) | exponent << 52 | (h & (1 << 52) - 1));
        let got = metallic::tanq(x as f128) as f64;
        let want = core_math::tan(x);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("tanq({x:e}) = {got:e} != {want:e}"))
    }));
}

#[test]
fn test_tanq_odd() {
    common::truncate_errors((0..SAMPLE_COUNT).filter_map(|i| {
        let x = wide(i);
        let (got, want) = (metallic::tanq(-x), -metallic::tanq(x));
        (!got.is(&want)).then(|| println!("tanq({:?}) = {got:?} != {want:?}", -x))
    }));
}

#[test]
fn test_tanq_special() {
    assert!(metallic::tanq(0.0).is(&0.0));
    assert!(metallic::tanq(-0.0).is(&-0.0));
    assert!(metallic::tanq(f128::INFINITY).is_nan());
    assert!(metallic::tanq(f128::NEG_INFINITY).is_nan());
    assert!(metallic::tanq(f128::NAN).is_nan());
    // tan(x) rounds back to x below the cubic term's reach.
    let least = f128::from_bits(1);
    assert!(metallic::tanq(least).is(&least));
    assert!(metallic::tanq(-least).is(&-least));
    assert!(metallic::tanq(f128::MIN_POSITIVE).is(&f128::MIN_POSITIVE));
    // The rounded π/4 is a hair under the true one: its tangent rounds to 1.
    assert!(metallic::tanq(core::f128::consts::FRAC_PI_4).is(&1.0));
    assert!(metallic::tanq(-core::f128::consts::FRAC_PI_4).is(&-1.0));
    // The rounded π/2 sits 2^-115.4 below the pole: a large finite tangent.
    assert!(metallic::tanq(core::f128::consts::FRAC_PI_2) > 1e33);
}

#[cfg(feature = "mpfr")]
fn oracle(x: f128) -> f128 {
    use rug::float::Round::Nearest;

    metallic::f128_mpfr::cr_unop(x, |y| y.tan_round(Nearest))
}

/// `round(k·π/2) + 2^-t` for `k < 2^30` and `t ∈ [7, 130]`: residuals from
/// the fast leg's whole range down past its hand-over to the accurate leg,
/// near poles (odd `k`) and zeros (even `k`) alike.
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
fn test_tanq_vs_mpfr() {
    for sampler in [direct, reduced, wide, near_multiple] {
        common128::mpfr_sweep_univariate_f128(metallic::tanq, oracle, sampler, SAMPLE_COUNT);
    }
}
