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

/// Every binary32 `(x, y)` whose exact `xʸ` is a rounding tie (issue #14):
/// 25-bit odd results in the normal range and odd multiples of `2⁻¹⁵⁰` in the
/// subnormal one.  No finite-precision fallback can decide these; they gate
/// the exact-case detector.  Regenerate with `python3 tools/gen_powf_ties.py`.
#[test]
fn test_powf_ties() {
    let cases: Vec<[f32; 2]> =
        common::parse_case_file("f32_powf.wc", common::parse_f32_pair).collect();
    assert_eq!(
        cases.len(),
        229_826,
        "corpus size changed; update this count"
    );
    common::test_bivariate_cases(metallic::powf, core_math::powf, cases.into_iter());
}
