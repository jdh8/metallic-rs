mod common;
use metallic::f32 as metal;

#[test]
fn test_parser() {
    assert!(common::parse_case_file("powf.wc", common::parse_f32_pair).count() == 133_216);
}

#[test]
fn test_powf() {
    common::test_bivariate_cases(
        metal::powf,
        core_math::powf,
        common::parse_case_file("powf.wc", common::parse_f32_pair),
    );
}
