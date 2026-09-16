use crate::{common, common128};
use common::Identity as _;

// CORE-MATH has no binary128 binding. The generated MPFR answers make this
// strict gate available without the mpfr feature.
const CORPUS_LEN: usize = 96_380;
const SAMPLES: u64 = 200_000;

#[test]
fn test_log2p1q_corpus() {
    let cases: Vec<[f128; 2]> =
        common::parse_case_file("log2p1q.wc", common128::parse_f128_pair).collect();
    assert_eq!(
        cases.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );
    common::truncate_errors(cases.into_iter().filter_map(|[x, want]| {
        let got = metallic::log2p1q(x);
        (!got.is(&want)).then(|| println!("log2p1q({x:?}) = {got:?} != {want:?}"))
    }));
}

#[test]
fn test_log2p1q_special() {
    for x in [0.0_f128, -0.0, f128::INFINITY] {
        assert!(metallic::log2p1q(x).is(&x));
    }
    assert!(metallic::log2p1q(-1.0).is(&f128::NEG_INFINITY));
    for x in [-2.0, f128::NEG_INFINITY, f128::NAN, (-1.0_f128).next_down()] {
        assert!(metallic::log2p1q(x).is_nan());
    }
    assert!(metallic::log2p1q(f128::MAX).is(&metallic::log2q(f128::MAX)));
    let least = f128::from_bits(1);
    for (x, want) in [(least, least), (-least, -least)] {
        assert!(metallic::log2p1q(x).is(&want));
    }
}

/// A correctly rounded f64 answer and a rounded f128 answer may differ by
/// one f64 ulp from double rounding; check all input binades without MPFR.
#[test]
fn test_log2p1q_vs_f64() {
    common::truncate_errors((0..SAMPLES).filter_map(|i| {
        let bits = common::mix64(i);
        let span = if bits >> 63 != 0 { 0x3ff } else { 0x7ff };
        let x =
            f64::from_bits(bits & 1 << 63 | ((bits >> 52) % span) << 52 | (bits & (1 << 52) - 1));
        let got = metallic::log2p1q(x as f128) as f64;
        let want = core_math::log2p1(x);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("log2p1q({x:e}) = {got:e} != {want:e}"))
    }));
}

#[cfg(feature = "mpfr")]
#[test]
fn test_log2p1q_vs_mpfr() {
    use super::log1pq::{dense, domain, near_minus_one, seams};
    use rug::float::Round::Nearest;
    let small = |i| {
        let bits = common128::mix128(i);
        let e = 16383 - 280 + (bits >> 112 & 0x7fff) % 262;
        f128::from_bits(bits & 1 << 127 | e << 112 | bits & (1 << 112) - 1)
    };
    let subnormal = |i| {
        let bits = common128::mix128(i);
        f128::from_bits(bits & 1 << 127 | bits & ((1 << 114) - 1))
    };
    for sampler in [domain, seams, near_minus_one, dense, small, subnormal] {
        common128::mpfr_sweep_univariate_f128(
            metallic::log2p1q,
            |x| metallic::f128_mpfr::cr_unop(x, |y| y.log2_1p_round(Nearest)),
            sampler,
            SAMPLES,
        );
    }
}

#[test]
fn test_log2p1q_exact() {
    for k in 1..=113 {
        let power = f128::from_bits(((16383 + k) as u128) << 112);
        assert_eq!(metallic::log2p1q(power - 1.0), k as f128);
        assert_eq!(metallic::log2p1q(1.0 / power - 1.0), -(k as f128));
    }
}
