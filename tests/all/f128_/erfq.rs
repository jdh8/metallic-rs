use crate::common128_special as common;

const CORPUS_LEN: usize = 213_981;

#[test]
fn test_erfq_corpus() {
    common::corpus(0, "erfq.wc", CORPUS_LEN);
}
#[test]
fn test_erfq_special() {
    common::special(0);
}
#[test]
fn test_erfq_vs_f64() {
    common::vs_f64(0);
}
#[test]
fn test_erfq_identities() {
    common::identities(0);
}
#[cfg(feature = "mpfr")]
#[test]
fn test_erfq_vs_mpfr() {
    common::vs_mpfr(0);
}
