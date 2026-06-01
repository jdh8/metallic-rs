mod common;
use metallic::f64 as metal;

#[test]
fn test_parser() {
    assert!(common::parse_case_file("cbrt.wc", common::parse_f64).count() == 105_554);
}

#[test]
fn test_cbrt() {
    common::test_univariate_cases(
        metal::cbrt,
        core_math::cbrt,
        common::parse_case_file("cbrt.wc", common::parse_f64)
            .chain((0..=u64::MAX).step_by((1 << 40) - 1337).map(f64::from_bits)),
    );
}
