use crate::common128_hyp as common;

const CORPUS_LEN: usize = 338_823;

#[test]
fn test_coshq_corpus() {
    common::corpus(1, "coshq.wc", CORPUS_LEN);
}
#[test]
fn test_coshq_special() {
    common::special(1);
}
#[test]
fn test_coshq_vs_f64() {
    common::vs_f64(1);
}
#[test]
fn test_coshq_symmetry_and_monotonicity() {
    common::symmetry_and_monotonicity(1);
}
#[cfg(feature = "mpfr")]
#[test]
fn test_coshq_vs_mpfr() {
    common::vs_mpfr(1);
}
