use crate::common128_hyp as common;

const CORPUS_LEN: usize = 460_992;

#[test]
fn test_sinhq_corpus() {
    common::corpus(0, "sinhq.wc", CORPUS_LEN);
}
#[test]
fn test_sinhq_special() {
    common::special(0);
}
#[test]
fn test_sinhq_vs_f64() {
    common::vs_f64(0);
}
#[test]
fn test_sinhq_symmetry_and_monotonicity() {
    common::symmetry_and_monotonicity(0);
}
#[cfg(feature = "mpfr")]
#[test]
fn test_sinhq_vs_mpfr() {
    common::vs_mpfr(0);
}
