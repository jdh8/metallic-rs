use crate::common128_hyp as common;

const CORPUS_LEN: usize = 300_932;

#[test]
fn test_tanhq_corpus() {
    common::corpus(2, "tanhq.wc", CORPUS_LEN);
}
#[test]
fn test_tanhq_special() {
    common::special(2);
}
#[test]
fn test_tanhq_vs_f64() {
    common::vs_f64(2);
}
#[test]
fn test_tanhq_symmetry_and_monotonicity() {
    common::symmetry_and_monotonicity(2);
}
#[cfg(feature = "mpfr")]
#[test]
fn test_tanhq_vs_mpfr() {
    common::vs_mpfr(2);
}
