use crate::common128_special as common;

const CORPUS_LEN: usize = 386_920;

#[test]
fn test_lgammaq_corpus() {
    common::corpus(3, "lgammaq.wc", CORPUS_LEN);
}
#[test]
fn test_lgammaq_special() {
    common::special(3);
}
#[test]
fn test_lgammaq_vs_f64() {
    common::vs_f64(3);
}
#[test]
fn test_lgammaq_identities() {
    common::identities(3);
}
#[cfg(feature = "mpfr")]
#[test]
fn test_lgammaq_vs_mpfr() {
    common::vs_mpfr(3);
}
