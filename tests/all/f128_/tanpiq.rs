use crate::common128_trigpi as common;

#[test]
fn test_tanpiq_corpus() {
    common::corpus("tanpiq", 179_436, 2);
}

#[test]
fn test_tanpiq_vs_f64() {
    common::cross_check(2);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_tanpiq_vs_mpfr() {
    common::sweep(2);
}

#[test]
fn test_tanpiq_symmetry() {
    common::symmetry(2);
}
