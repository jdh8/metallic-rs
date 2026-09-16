use crate::{common, common128};
use common::Identity as _;

const CORPUS_LEN: usize = 292_820;
pub(super) const SAMPLES: u64 = 200_000;

pub(super) fn corpus(name: &str, count: usize, f: fn(f128) -> f128) {
    assert!(count > 0, "a generated corpus is required");
    let cases: Vec<[f128; 2]> = common::parse_case_file(name, common128::parse_f128_pair).collect();
    assert_eq!(cases.len(), count, "corpus size changed; update this count");
    common::truncate_errors(cases.into_iter().filter_map(|[x, want]| {
        let got = f(x);
        (!got.is(&want)).then(|| println!("{name}: f({x:?}) = {got:?} != {want:?}"))
    }));
}

pub(super) fn special(f: fn(f128) -> f128, least: u128) {
    for x in [0.0_f128, -0.0, f128::INFINITY] {
        assert!(f(x).is(&x));
    }
    for x in [f128::NEG_INFINITY, -f128::MAX, -128.0] {
        assert!(f(x).is(&-1.0));
    }
    assert!(f(f128::MAX).is(&f128::INFINITY));
    for bits in [
        0x7fff_u128 << 112 | 1,
        0xffff_u128 << 112 | 7,
        f128::NAN.to_bits(),
    ] {
        assert_eq!(f(f128::from_bits(bits)).to_bits(), bits | 1 << 111);
    }
    assert_eq!(f(f128::from_bits(1)).to_bits(), least);
    assert_eq!(f(-f128::from_bits(1)).to_bits(), 1 << 127 | least);
}

pub(super) fn vs_f64(f: fn(f128) -> f128, oracle: fn(f64) -> f64) {
    common::truncate_errors((0..SAMPLES).filter_map(|i| {
        let x = f64::from_bits(common::mix64(i));
        let got = f(x as f128) as f64;
        let want = oracle(x);
        (common::ulp_error_f64(got, want) > 1).then(|| println!("f({x:e}) = {got:e} != {want:e}"))
    }));
}

#[test]
fn test_exp2m1q_corpus() {
    corpus("exp2m1q.wc", CORPUS_LEN, metallic::exp2m1q);
}

#[test]
fn test_exp2m1q_special() {
    special(metallic::exp2m1q, 1);
}

#[test]
fn test_exp2m1q_vs_f64() {
    vs_f64(metallic::exp2m1q, core_math::exp2m1);
}

#[test]
fn test_exp2m1q_exact_and_midpoint() {
    for k in 1..=113 {
        let p = f128::from_bits(((16383 + k) as u128) << 112);
        assert_eq!(metallic::exp2m1q(k as f128), p - 1.0);
        assert_eq!(metallic::exp2m1q(-(k as f128)), 1.0 / p - 1.0);
    }
    // 2^114 - 1 and -1 + 2^-114 are exact midpoints; both even endpoints
    // round away from zero. The adjacent inputs must reach either side.
    let p = f128::from_bits((16383 + 114) << 112);
    assert_eq!(metallic::exp2m1q(114.0), p);
    assert_eq!(metallic::exp2m1q(-114.0), -1.0);
    assert!(metallic::exp2m1q(114.0_f128.next_down()) < p);
    assert_eq!(
        metallic::exp2m1q((-114.0_f128).next_up()),
        (-1.0_f128).next_up()
    );
    assert!(metallic::exp2m1q(16384.0).is_infinite());
    assert!(metallic::exp2m1q(16384.0_f128.next_down()).is_finite());
}

#[cfg(feature = "mpfr")]
pub(super) fn domain(i: u64) -> f128 {
    f128::from_bits(common128::mix128(i))
}

#[cfg(feature = "mpfr")]
pub(super) fn band(i: u64) -> f128 {
    let bits = common128::mix128(i);
    let e = 16383 - 400 + (i % 415) as u128;
    f128::from_bits(bits & 1 << 127 | e << 112 | bits & (1 << 112) - 1)
}

#[cfg(feature = "mpfr")]
pub(super) fn subnormal(i: u64) -> f128 {
    let bits = common128::mix128(i);
    f128::from_bits(bits & 1 << 127 | bits & (1 << 114) - 1)
}

#[cfg(feature = "mpfr")]
pub(super) fn seams(i: u64) -> f128 {
    let anchors: [i32; 9] = [-385, -384, -273, -272, -129, -128, -20, -19, -6];
    let anchor = ((16383 + anchors[(i / 2 % 9) as usize]) as u128) << 112;
    let offset = (i / 18) as i128 - (SAMPLES / 36) as i128;
    f128::from_bits(u128::from(i % 2) << 127 | (anchor as i128 + offset) as u128)
}

#[cfg(feature = "mpfr")]
#[test]
fn test_exp2m1q_vs_mpfr() {
    use rug::float::Round::Nearest;
    for sampler in [domain, band, subnormal, seams] {
        common128::mpfr_sweep_univariate_f128(
            metallic::exp2m1q,
            |x| metallic::f128_mpfr::cr_unop(x, |y| y.exp2_m1_round(Nearest)),
            sampler,
            SAMPLES,
        );
    }
    common::test_univariate_cases(
        metallic::exp2m1q,
        |x| metallic::f128_mpfr::cr_unop(x, |y| y.exp2_m1_round(Nearest)),
        crate::common_exp::dense(-115.0, 16400.0),
    );
}
