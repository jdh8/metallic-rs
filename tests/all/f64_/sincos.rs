use crate::common;

#[test]
fn test_parser() {
    assert!(common::parse_case_file("sincos.wc", common::parse_f64).count() == 81_551);
}

#[test]
fn test_sin_cos() {
    let dense = (0..=2_000_000).map(|i| metallic::fma(f64::from(i), 20.0 / 2_000_000.0, -10.0));
    let bits = (0..=u64::MAX).step_by((1 << 39) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metallic::sincos, core_math::sincos, dense.chain(bits));
}

/// Correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_sincos_worst_cases() {
    common::test_worst_univariate("sincos", metallic::sincos, core_math::sincos);
}

/// Faithful-rounding floor (≤ 1 ulp on each of sin and cos) on that same corpus.
#[test]
fn test_sincos_worst_faithful() {
    common::truncate_errors(
        common::parse_case_file("sincos.wc", common::parse_f64).filter_map(|x| {
            let (s, c) = metallic::sincos(x);
            let (rs, rc) = core_math::sincos(x);
            let (es, ec) = (common::ulp_error_f64(s, rs), common::ulp_error_f64(c, rc));
            (es > 1 || ec > 1).then(|| println!("sincos({x:e}): sin {es} ulp, cos {ec} ulp"))
        }),
    );
}
