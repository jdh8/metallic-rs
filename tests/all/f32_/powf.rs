use crate::common;

#[test]
fn test_parser() {
    assert!(common::parse_case_file("powf.wc", common::parse_f32_pair).count() == 404_160);
}

#[test]
fn test_powf() {
    common::test_bivariate_cases(
        metallic::powf,
        core_math::powf,
        common::parse_case_file("powf.wc", common::parse_f32_pair),
    );
}
