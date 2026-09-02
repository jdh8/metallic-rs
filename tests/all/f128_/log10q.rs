use crate::common;
use crate::common128;

use common::Identity as _;

// CORE-MATH has not shipped `log10q` yet, so the strict gate replays a
// home-grown corpus that carries its own MPFR answers
// (`examples/gen_f128_log_cases.rs`) and runs under plain `--features f128`.
// The parser count keeps the corpus from rotting unnoticed, the f64 oracle
// covers every binade with no MPFR, and the MPFR sweeps stay the independent
// cross-check.

/// Size of `tests/cases/log10q.wc` (kept in sync with the generator).
const CORPUS_LEN: usize = 45_544;

const SAMPLE_COUNT: u64 = 200_000;

/// A random significand at an exponent uniform over every binade, subnormals
/// included: the whole domain of the logarithm.
#[cfg(feature = "mpfr")]
fn positive(i: u64) -> f128 {
    let bits = common128::mix128(i);
    let exponent = (bits >> 120) % 0x7fff;

    f128::from_bits(exponent << 112 | (bits & (1 << 112) - 1))
}

/// Bit-stepping sweep of the neighbourhood of 1, where the reduction cancels
/// and the accurate leg decides alone.
#[cfg(feature = "mpfr")]
fn near_one(i: u64) -> f128 {
    let offset = u128::from(i >> 1);
    let bits = 1.0_f128.to_bits();

    f128::from_bits(if i & 1 == 0 {
        bits + offset
    } else {
        bits - offset
    })
}

/// Uniform dense sweep of `(0, 2]`.
#[cfg(feature = "mpfr")]
fn dense(i: u64) -> f128 {
    2.0 * (i + 1) as f128 / SAMPLE_COUNT as f128
}

#[test]
fn test_log10q_corpus() {
    let cases: Vec<[f128; 2]> =
        common::parse_case_file("log10q.wc", common128::parse_f128_pair).collect();
    assert_eq!(
        cases.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );

    common::truncate_errors(cases.into_iter().filter_map(|[x, want]| {
        let got = metallic::log10q(x);
        (!got.is(&want)).then(|| println!("log10q({x:?}) = {got:?} != {want:?} (correct)"))
    }));
}

/// Every `f64` is a binary128 value, and its correctly rounded `f64` log10 can
/// differ from the binary128 one rounded down to `f64` by at most one ulp
/// (the double rounding).  With CORE-MATH's `log10` as the oracle this checks
/// every binade of the domain with no MPFR at all.
#[test]
fn test_log10q_vs_f64() {
    common::truncate_errors((0..SAMPLE_COUNT).filter_map(|i| {
        let h = common::mix64(i);
        let x = f64::from_bits(((h >> 52) % 0x7ff) << 52 | (h & (1 << 52) - 1));
        let got = metallic::log10q(x as f128) as f64;
        let want = core_math::log10(x);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("log10q({x:e}) = {got:e} != {want:e}"))
    }));
}

/// `log10(10^k) = k` exactly for every representable power of ten:
/// `5^48 < 2^112 < 5^49`.
#[test]
fn test_log10q_exact() {
    let powers = std::iter::successors(Some(1.0_f128), |x| Some(x * 10.0)).take(49);

    common::truncate_errors(powers.enumerate().filter_map(|(k, x)| {
        let got = metallic::log10q(x);
        (!got.is(&(k as f128))).then(|| println!("log10q(10^{k}) = {got:?}"))
    }));
}

#[test]
fn test_log10q_special() {
    assert!(metallic::log10q(1.0).is(&0.0));
    assert!(metallic::log10q(0.0).is(&f128::NEG_INFINITY));
    assert!(metallic::log10q(-0.0).is(&f128::NEG_INFINITY));
    assert!(metallic::log10q(f128::INFINITY).is(&f128::INFINITY));
    assert!(metallic::log10q(-1.0).is_nan());
    assert!(metallic::log10q(f128::NEG_INFINITY).is_nan());
    assert!(metallic::log10q(f128::NAN).is_nan());
    assert!(metallic::log10q(10.0).is(&1.0));
    assert!(metallic::log10q(2.0).is(&core::f128::consts::LOG10_2));
}

#[cfg(feature = "mpfr")]
#[test]
fn test_log10q_vs_mpfr() {
    use rug::float::Round::Nearest;

    for sampler in [positive, near_one, dense] {
        common128::mpfr_sweep_univariate_f128(
            metallic::log10q,
            |x| metallic::f128_mpfr::cr_unop(x, |y| y.log10_round(Nearest)),
            sampler,
            SAMPLE_COUNT,
        );
    }
}
