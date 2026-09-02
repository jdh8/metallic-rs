use crate::common;

#[test]
fn test_atan2() {
    // The worst-case file lists `y,x` pairs, matching the `atan2(y, x)` order.
    common::test_bivariate_cases(
        metallic::atan2f,
        core_math::atan2f,
        common::parse_case_file("atan2f.wc", common::parse_f32_pair),
    );
}
