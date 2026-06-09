mod common;

#[test]
fn test_asin() {
    let dense = (0..=4_000_000).map(|i| metallic::fma(f64::from(i), 2.0 / 4_000_000.0, -1.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    // CORE-MATH's published hard-to-round asin arguments (and their negatives, by
    // symmetry), which the dense/bit sweeps do not hit exactly.
    let hard = ["0x1.fffffffffffffp-1", "0x1.fffffffffffffp-7"]
        .into_iter()
        .flat_map(|s| {
            let x = common::parse_f64(s).unwrap();
            [x, -x]
        });
    common::test_univariate_cases(
        metallic::asin,
        core_math::asin,
        dense.chain(bits).chain(hard),
    );
}
