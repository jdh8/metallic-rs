mod common;

#[test]
fn test_atan2() {
    // A dense (y, x) grid over [-5, 5]² plus hashed wide-magnitude pairs and the
    // ∞/0/sign special cases.
    let grid = (0..2500u64).flat_map(|i| {
        (0..2500u64).map(move |j| {
            let y = metallic::fma(f64::from(i as u32), 10.0 / 2500.0, -5.0);
            let x = metallic::fma(f64::from(j as u32), 10.0 / 2500.0, -5.0);
            [y, x]
        })
    });
    let wide = (0..2_000_000u64).map(|i| {
        let y = f64::from_bits(i.wrapping_mul(0x2545_F491_4F6C_DD1D));
        let x = f64::from_bits(i.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xDEAD);
        [y, x]
    });
    common::test_bivariate_cases(metallic::atan2, core_math::atan2, grid.chain(wide));
}
