use crate::common;
use crate::common128;

use common::Identity as _;

// CORE-MATH has not shipped `log1pq` yet, so the strict gate replays a
// home-grown corpus that carries its own MPFR answers
// (`examples/gen_f128_log_cases.rs -- 1p`) and runs under plain
// `--features f128`.  The parser count keeps the corpus from rotting
// unnoticed, the f64 oracle covers every binade with no MPFR, and the MPFR
// sweeps stay the independent cross-check.

/// Size of `tests/cases/log1pq.wc` (kept in sync with the generator).
const CORPUS_LEN: usize = 69_913;

const SAMPLE_COUNT: u64 = 200_000;

/// The whole domain: a random significand at an exponent uniform over every
/// binade, subnormals included — positive, or negative above −1.
#[cfg(feature = "mpfr")]
fn domain(i: u64) -> f128 {
    let bits = common128::mix128(i);
    let span = if bits >> 127 != 0 { 0x3fff } else { 0x7fff };
    let exponent = (bits >> 112 & 0x7fff) % span;

    f128::from_bits(bits & 1 << 127 | exponent << 112 | (bits & (1 << 112) - 1))
}

/// Bit-stepping sweeps around the seams, both signs: `2^-113`, below which
/// the result is the argument; `2^-112`, where `x²/2` is exactly half an
/// ulp; and `2^-18`, where the floating leg hands over to the reduction.
#[cfg(feature = "mpfr")]
fn seams(i: u64) -> f128 {
    let anchors: [u128; 3] = [16383 - 113, 16383 - 112, 16383 - 18];
    let anchor = anchors[(i % 6 / 2) as usize] << 112;
    let offset = (i / 6) as i128 - (SAMPLE_COUNT / 12) as i128;
    let magnitude = (anchor as i128 + offset) as u128;

    f128::from_bits(u128::from(i % 2) << 127 | magnitude)
}

/// `−1 + t` for `t` log-uniform over `[2^-113, 2^-1)`: `1 + x` small and
/// exact, the reduction's cancellation.
#[cfg(feature = "mpfr")]
fn near_minus_one(i: u64) -> f128 {
    let bits = common128::mix128(i);
    let exponent = 16383 - 113 + (bits >> 112 & 0x7fff) % 113;

    -1.0 + f128::from_bits(exponent << 112 | (bits & (1 << 112) - 1))
}

/// Uniform dense sweep of `(−1, 2]`.
#[cfg(feature = "mpfr")]
fn dense(i: u64) -> f128 {
    3.0 * (i + 1) as f128 / SAMPLE_COUNT as f128 - 1.0
}

#[test]
fn test_log1pq_corpus() {
    let cases: Vec<[f128; 2]> =
        common::parse_case_file("log1pq.wc", common128::parse_f128_pair).collect();
    assert_eq!(
        cases.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );

    common::truncate_errors(cases.into_iter().filter_map(|[x, want]| {
        let got = metallic::log1pq(x);
        (!got.is(&want)).then(|| println!("log1pq({x:?}) = {got:?} != {want:?} (correct)"))
    }));
}

/// Every `f64` is a binary128 value, and its correctly rounded `f64` log1p
/// can differ from the binary128 one rounded down to `f64` by at most one ulp
/// (the double rounding).  With CORE-MATH's `log1p` as the oracle this checks
/// every binade of the domain with no MPFR at all.
#[test]
fn test_log1pq_vs_f64() {
    common::truncate_errors((0..SAMPLE_COUNT).filter_map(|i| {
        let h = common::mix64(i);
        let span = if h >> 63 != 0 { 0x3ff } else { 0x7ff };
        let x = f64::from_bits(h & 1 << 63 | ((h >> 52) % span) << 52 | (h & (1 << 52) - 1));
        let got = metallic::log1pq(x as f128) as f64;
        let want = core_math::log1p(x);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("log1pq({x:e}) = {got:e} != {want:e}"))
    }));
}

/// Below `|x| = 2^-113` the answer is `x` itself, subnormals included.
#[test]
fn test_log1pq_tiny() {
    common::truncate_errors((0..SAMPLE_COUNT).filter_map(|i| {
        let bits = common128::mix128(i);
        let exponent = (bits >> 112 & 0x7fff) % (16383 - 113);
        let x = f128::from_bits(bits & 1 << 127 | exponent << 112 | (bits & (1 << 112) - 1));
        let got = metallic::log1pq(x);
        (!got.is(&x)).then(|| println!("log1pq({x:?}) = {got:?}"))
    }));
}

#[test]
fn test_log1pq_special() {
    assert!(metallic::log1pq(0.0).is(&0.0));
    assert!(metallic::log1pq(-0.0).is(&-0.0));
    assert!(metallic::log1pq(-1.0).is(&f128::NEG_INFINITY));
    assert!(metallic::log1pq(-2.0).is_nan());
    assert!(metallic::log1pq(f128::NEG_INFINITY).is_nan());
    assert!(metallic::log1pq(f128::INFINITY).is(&f128::INFINITY));
    assert!(metallic::log1pq(f128::NAN).is_nan());
    assert!(metallic::log1pq(1.0).is(&core::f128::consts::LN_2));
    assert!(metallic::log1pq(-0.5).is(&-core::f128::consts::LN_2));
    assert!(metallic::log1pq(3.0).is(&(2.0 * core::f128::consts::LN_2)));
    assert!(metallic::log1pq(f128::MAX).is(&metallic::logq(f128::MAX)));
}

#[cfg(feature = "mpfr")]
#[test]
fn test_log1pq_vs_mpfr() {
    use rug::float::Round::Nearest;

    for sampler in [domain, seams, near_minus_one, dense] {
        common128::mpfr_sweep_univariate_f128(
            metallic::log1pq,
            |x| metallic::f128_mpfr::cr_unop(x, |y| y.ln_1p_round(Nearest)),
            sampler,
            SAMPLE_COUNT,
        );
    }
}
