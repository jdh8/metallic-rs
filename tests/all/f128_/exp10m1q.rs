use super::exp2m1q;

const CORPUS_LEN: usize = 295_132;

#[test]
fn test_exp10m1q_corpus() {
    exp2m1q::corpus("exp10m1q.wc", CORPUS_LEN, metallic::exp10m1q);
}

#[test]
fn test_exp10m1q_special() {
    exp2m1q::special(metallic::exp10m1q, 2);
}

#[test]
fn test_exp10m1q_vs_f64() {
    exp2m1q::vs_f64(metallic::exp10m1q, core_math::exp10m1);
}

#[test]
fn test_exp10m1q_exact() {
    let mut power = 1_u128;
    for k in 1..=34 {
        power *= 10;
        assert_eq!(metallic::exp10m1q(k as f128), (power - 1) as f128);
    }

    // 5^49 has 114 bits: 10^49 itself is a binary128 midpoint, and
    // subtracting one puts the true result just below it (49 bits below
    // the rounding bit). Construct the lower neighbour as an exact integer.
    let significand = 5_u128.pow(49) >> 1;
    let want = f128::from_bits(((16383 + 162) << 112) | (significand - (1 << 112)));
    assert_eq!(metallic::exp10m1q(49.0), want);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_exp10m1q_vs_mpfr() {
    use crate::{common, common128};
    use rug::float::Round::Nearest;
    for sampler in [
        exp2m1q::domain,
        exp2m1q::band,
        exp2m1q::subnormal,
        exp2m1q::seams,
    ] {
        common128::mpfr_sweep_univariate_f128(
            metallic::exp10m1q,
            |x| metallic::f128_mpfr::cr_unop(x, |y| y.exp10_m1_round(Nearest)),
            sampler,
            exp2m1q::SAMPLES,
        );
    }
    common::test_univariate_cases(
        metallic::exp10m1q,
        |x| metallic::f128_mpfr::cr_unop(x, |y| y.exp10_m1_round(Nearest)),
        crate::common_exp::dense(-36.0, 4940.0),
    );
}
