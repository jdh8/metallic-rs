use crate::common128_special as common;

const CORPUS_LEN: usize = 339_182;

#[test]
fn test_erfcq_corpus() {
    common::corpus(1, "erfcq.wc", CORPUS_LEN);
}
#[test]
fn test_erfcq_special() {
    common::special(1);
}
#[test]
fn test_erfcq_vs_f64() {
    common::vs_f64(1);
}
#[test]
fn test_erfcq_identities() {
    common::identities(1);
}
#[cfg(feature = "mpfr")]
#[test]
fn test_erfcq_vs_mpfr() {
    common::vs_mpfr(1);
}
