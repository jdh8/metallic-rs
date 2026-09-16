use crate::common128_invhyp as common;

const CORPUS_LEN: usize = 713_262;

#[test]
fn test_asinhq_corpus() {
    common::corpus(0, "asinhq.wc", CORPUS_LEN);
}
#[test]
fn test_asinhq_special() {
    common::special(0);
}
#[test]
fn test_asinhq_vs_f64() {
    common::vs_f64(0);
}
#[test]
fn test_asinhq_symmetry_and_monotonicity() {
    common::symmetry_and_monotonicity(0);
}
#[cfg(feature = "mpfr")]
#[test]
fn test_asinhq_vs_mpfr() {
    common::vs_mpfr(0);
}
