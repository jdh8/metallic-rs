use crate::common;
use crate::common128;

use common::Identity as _;

// CORE-MATH has not shipped `log2q` yet, so the strict gate replays a
// home-grown corpus that carries its own MPFR answers
// (`examples/gen_f128_log_cases.rs`) and runs under plain `--features f128`.
// The parser count keeps the corpus from rotting unnoticed, the f64 oracle
// covers every binade with no MPFR, and the MPFR sweeps stay the independent
// cross-check.

/// Size of `tests/cases/log2q.wc` (kept in sync with the generator).
const CORPUS_LEN: usize = 45_520;

const SAMPLE_COUNT: u64 = 200_000;

/// A random significand at an exponent uniform over every binade, subnormals
/// included: the whole domain of the logarithm.
#[cfg(feature = "mpfr")]
fn positive(i: u64) -> f128 {
    let bits = common128::mix128(i);
    let exponent = (bits >> 112 & 0x7fff) % 0x7fff;

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
fn test_log2q_corpus() {
    let cases: Vec<[f128; 2]> =
        common::parse_case_file("log2q.wc", common128::parse_f128_pair).collect();
    assert_eq!(
        cases.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );

    common::truncate_errors(cases.into_iter().filter_map(|[x, want]| {
        let got = metallic::log2q(x);
        (!got.is(&want)).then(|| println!("log2q({x:?}) = {got:?} != {want:?} (correct)"))
    }));
}

/// Every `f64` is a binary128 value, and its correctly rounded `f64` log2 can
/// differ from the binary128 one rounded down to `f64` by at most one ulp
/// (the double rounding).  With CORE-MATH's `log2` as the oracle this checks
/// every binade of the domain with no MPFR at all.
#[test]
fn test_log2q_vs_f64() {
    common::truncate_errors((0..SAMPLE_COUNT).filter_map(|i| {
        let h = common::mix64(i);
        let x = f64::from_bits(((h >> 52) % 0x7ff) << 52 | (h & (1 << 52) - 1));
        let got = metallic::log2q(x as f128) as f64;
        let want = core_math::log2(x);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("log2q({x:e}) = {got:e} != {want:e}"))
    }));
}

/// `log2(2^k) = k` exactly, for every exponent down through the subnormals.
#[test]
fn test_log2q_exact() {
    common::truncate_errors((-16494..=16383).filter_map(|k| {
        let x = f128::from_bits(if k < -16382 {
            1 << (k + 16494)
        } else {
            ((k + 16383) as u128) << 112
        });
        let got = metallic::log2q(x);
        (!got.is(&(k as f128))).then(|| println!("log2q(2^{k}) = {got:?}"))
    }));
}

#[test]
fn test_log2q_special() {
    assert!(metallic::log2q(1.0).is(&0.0));
    assert!(metallic::log2q(0.0).is(&f128::NEG_INFINITY));
    assert!(metallic::log2q(-0.0).is(&f128::NEG_INFINITY));
    assert!(metallic::log2q(f128::INFINITY).is(&f128::INFINITY));
    assert!(metallic::log2q(-1.0).is_nan());
    assert!(metallic::log2q(f128::NEG_INFINITY).is_nan());
    assert!(metallic::log2q(f128::NAN).is_nan());
    assert!(metallic::log2q(f128::from_bits(1)).is(&-16494.0));
    assert!(metallic::log2q(f128::MIN_POSITIVE).is(&-16382.0));
}

#[cfg(feature = "mpfr")]
#[test]
fn test_log2q_vs_mpfr() {
    use rug::float::Round::Nearest;

    for sampler in [positive, near_one, dense] {
        common128::mpfr_sweep_univariate_f128(
            metallic::log2q,
            |x| metallic::f128_mpfr::cr_unop(x, |y| y.log2_round(Nearest)),
            sampler,
            SAMPLE_COUNT,
        );
    }
}
