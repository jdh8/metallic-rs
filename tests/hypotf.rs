mod common;

#[test]
fn test_parser() {
    assert!(common::parse_case_file("hypotf.wc", common::parse_f32_pair).count() == 6882);
}

#[test]
fn test_hypot() {
    common::test_bivariate_cases(
        metallic::hypotf,
        core_math::hypotf,
        common::parse_case_file("hypotf.wc", common::parse_f32_pair),
    );
}
