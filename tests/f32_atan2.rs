mod common;
use metallic::f32 as metal;

#[test]
fn test_atan2() {
    // The worst-case file lists `y,x` pairs, matching the `atan2(y, x)` order.
    common::test_bivariate_cases(
        metal::atan2,
        core_math::atan2f,
        common::parse_case_file("atan2f.wc", common::parse_f32_pair),
    );
}
