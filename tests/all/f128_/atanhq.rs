use crate::common128_invhyp as common;

const CORPUS_LEN: usize = 698_150;

#[test]
fn test_atanhq_corpus() {
    common::corpus(2, "atanhq.wc", CORPUS_LEN);
}
#[test]
fn test_atanhq_special() {
    common::special(2);
}
#[test]
fn test_atanhq_vs_f64() {
    common::vs_f64(2);
}
#[test]
fn test_atanhq_symmetry_and_monotonicity() {
    common::symmetry_and_monotonicity(2);
}
#[cfg(feature = "mpfr")]
#[test]
fn test_atanhq_vs_mpfr() {
    common::vs_mpfr(2);
}
