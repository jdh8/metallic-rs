use crate::common;
use crate::common128;

use common::Identity as _;

// `sincosq` is `sinq` and `cosq` off one Payne–Hanek reduction and one pair of
// series, and both legs behind it are correctly rounded, so each half must
// equal the separate function bit for bit.  That identity is the gate: it
// inherits `sinq`'s and `cosq`'s own corpora (which carry their MPFR answers)
// through the halves checked here, and the joint Ziv hand-over — either half
// landing in the tie window sends *both* to the 384-bit leg — is exactly what
// a shared-reduction pair can get wrong.

/// Sizes of `tests/cases/{sinq,cosq}.wc` (kept in sync with the generators).
const SIN_CORPUS_LEN: usize = 95_321;
const COS_CORPUS_LEN: usize = 96_265;

const SAMPLE_COUNT: u64 = 200_000;

/// Representation-uniform bits: every binade, subnormals, and the specials.
fn wide(i: u64) -> f128 {
    f128::from_bits(common128::mix128(i))
}

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

/// `round(k·π/2) + 2^-t` for `k < 2^30` and `t ∈ [7, 130]`: residuals from the
/// fast leg's whole range down past its hand-over to the accurate leg, where
/// the two halves are most likely to disagree about the gate.
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

/// The sine half against `sinq`'s corpus of MPFR answers.
#[test]
fn test_sincosq_sin_corpus() {
    let cases: Vec<[f128; 2]> =
        common::parse_case_file("sinq.wc", common128::parse_f128_pair).collect();
    assert_eq!(
        cases.len(),
        SIN_CORPUS_LEN,
        "corpus size changed; update this count"
    );

    common::truncate_errors(cases.into_iter().filter_map(|[x, want]| {
        let (got, _) = metallic::sincosq(x);
        (!got.is(&want)).then(|| println!("sincosq({x:?}).0 = {got:?} != {want:?} (correct)"))
    }));
}

/// The cosine half against `cosq`'s corpus of MPFR answers.
#[test]
fn test_sincosq_cos_corpus() {
    let cases: Vec<[f128; 2]> =
        common::parse_case_file("cosq.wc", common128::parse_f128_pair).collect();
    assert_eq!(
        cases.len(),
        COS_CORPUS_LEN,
        "corpus size changed; update this count"
    );

    common::truncate_errors(cases.into_iter().filter_map(|[x, want]| {
        let (_, got) = metallic::sincosq(x);
        (!got.is(&want)).then(|| println!("sincosq({x:?}).1 = {got:?} != {want:?} (correct)"))
    }));
}

/// The pair is the two separate functions, everywhere.
#[test]
fn test_sincosq_agrees() {
    for sampler in [direct, reduced, wide] {
        common::test_univariate_cases(
            metallic::sincosq,
            |x| (metallic::sinq(x), metallic::cosq(x)),
            (0..SAMPLE_COUNT).map(sampler),
        );
    }
}

/// The identity again where the Ziv gates are, and against MPFR by way of
/// `sinq`/`cosq`, whose own sweeps certify them there.
#[cfg(feature = "mpfr")]
#[test]
fn test_sincosq_agrees_near_multiples() {
    common::test_univariate_cases(
        metallic::sincosq,
        |x| (metallic::sinq(x), metallic::cosq(x)),
        (0..SAMPLE_COUNT).map(near_multiple),
    );
}

/// Every `f64` is a binary128 value, so CORE-MATH's `sin` and `cos` check both
/// halves at every magnitude up to 2^1024 with no MPFR at all; the double
/// rounding down to `f64` costs at most one ulp.
#[test]
fn test_sincosq_vs_f64() {
    common::truncate_errors((0..SAMPLE_COUNT).flat_map(|i| {
        let h = common::mix64(i);
        let exponent = 1023 - 57 + (h >> 52 & 0x7ff) % 1081;
        let x = f64::from_bits((h & 1 << 63) | exponent << 52 | (h & (1 << 52) - 1));
        let (sin, cos) = metallic::sincosq(x as f128);
        let (sin, cos) = (sin as f64, cos as f64);
        let (ws, wc) = (core_math::sin(x), core_math::cos(x));

        [(sin, ws, "sin"), (cos, wc, "cos")]
            .into_iter()
            .filter_map(move |(got, want, half)| {
                (common::ulp_error_f64(got, want) > 1)
                    .then(|| println!("sincosq({x:e}).{half} = {got:e} != {want:e}"))
            })
    }));
}

#[test]
fn test_sincosq_parity() {
    common::truncate_errors((0..SAMPLE_COUNT).filter_map(|i| {
        let x = wide(i);
        let (sin, cos) = metallic::sincosq(x);
        let (nsin, ncos) = metallic::sincosq(-x);
        (!nsin.is(&-sin) || !ncos.is(&cos))
            .then(|| println!("sincosq({:?}) = {nsin:?}, {ncos:?} breaks parity", -x))
    }));
}

#[test]
fn test_sincosq_special() {
    let is = |(s, c): (f128, f128), (ws, wc): (f128, f128)| s.is(&ws) && c.is(&wc);

    assert!(is(metallic::sincosq(0.0), (0.0, 1.0)));
    assert!(is(metallic::sincosq(-0.0), (-0.0, 1.0)));
    assert!(metallic::sincosq(core::f128::consts::FRAC_PI_2).0.is(&1.0));
    assert!(metallic::sincosq(core::f128::consts::PI).1.is(&-1.0));
    for x in [f128::INFINITY, f128::NEG_INFINITY, f128::NAN] {
        let (sin, cos) = metallic::sincosq(x);
        assert!(sin.is_nan() && cos.is_nan());
    }
    // Below 2^-57 the sine is the argument and the cosine one, signed zeros
    // and subnormals included.
    for x in [f128::from_bits(1), f128::MIN_POSITIVE] {
        assert!(is(metallic::sincosq(x), (x, 1.0)));
        assert!(is(metallic::sincosq(-x), (-x, 1.0)));
    }
}
