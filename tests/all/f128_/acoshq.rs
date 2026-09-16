use crate::common128_invhyp as common;

const CORPUS_LEN: usize = 387_906;

#[test]
fn test_acoshq_corpus() {
    common::corpus(1, "acoshq.wc", CORPUS_LEN);
}
#[test]
fn test_acoshq_special() {
    common::special(1);
}
#[test]
fn test_acoshq_vs_f64() {
    common::vs_f64(1);
}
#[test]
fn test_acoshq_symmetry_and_monotonicity() {
    common::symmetry_and_monotonicity(1);
}
#[cfg(feature = "mpfr")]
#[test]
fn test_acoshq_vs_mpfr() {
    common::vs_mpfr(1);
}
