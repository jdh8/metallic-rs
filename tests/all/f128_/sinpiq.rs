use crate::common128_trigpi as common;

#[test]
fn test_sinpiq_corpus() {
    common::corpus("sinpiq", 200_895, 0);
}

#[test]
fn test_sinpiq_vs_f64() {
    common::cross_check(0);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_sinpiq_vs_mpfr() {
    common::sweep(0);
}

#[test]
fn test_sinpiq_symmetry() {
    common::symmetry(0);
}
