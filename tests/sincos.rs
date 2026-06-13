mod common;

#[test]
fn test_parser() {
    assert!(common::parse_case_file("sincos.wc", common::parse_f64).count() == 81_551);
}

#[test]
fn test_sin_cos() {
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 20.0 / 2_000_000.0, -10.0));
    let bits = (0..=u64::MAX).step_by((1 << 39) - 1337).map(f64::from_bits);
    common::test_univariate_cases(
        metallic::sincos,
        core_math::sincos,
        common::parse_case_file("sincos.wc", common::parse_f64)
            .chain(dense)
            .chain(bits),
    );
}
