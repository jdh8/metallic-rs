use crate::common128_trigpi as common;

#[test]
fn test_cospiq_corpus() {
    common::corpus("cospiq", 95_888, 1);
}

#[test]
fn test_cospiq_vs_f64() {
    common::cross_check(1);
}

#[cfg(feature = "mpfr")]
#[test]
fn test_cospiq_vs_mpfr() {
    common::sweep(1);
}

#[test]
fn test_cospiq_symmetry() {
    common::symmetry(1);
}
