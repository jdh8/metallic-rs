use crate::common128_special as common;

const CORPUS_LEN: usize = 364_217;

#[test]
fn test_tgammaq_corpus() {
    common::corpus(2, "tgammaq.wc", CORPUS_LEN);
}
#[test]
fn test_tgammaq_special() {
    common::special(2);
}
#[test]
fn test_tgammaq_vs_f64() {
    common::vs_f64(2);
}
#[test]
fn test_tgammaq_identities() {
    common::identities(2);
}
#[cfg(feature = "mpfr")]
#[test]
fn test_tgammaq_vs_mpfr() {
    common::vs_mpfr(2);
}
