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

#[test]
fn test_atan2_grid_and_full_range() {
    let grid = (0..2000_u64).flat_map(|i| {
        (0..2000_u64).map(move |j| {
            [
                metallic::fmaf(i as f32, 0.001, -1.0),
                metallic::fmaf(j as f32, 0.001, -1.0),
            ]
        })
    });
    let wide = (0..4_000_000_u64).map(|i| {
        let h = common::mix64(i);
        [f32::from_bits(h as u32), f32::from_bits((h >> 32) as u32)]
    });
    common::test_bivariate_cases(metallic::atan2f, core_math::atan2f, grid.chain(wide));
}

/// All sign combinations across the axes, diagonal, subnormal rounding, and
/// extreme exponent gaps; both unit systems share the new finite-input leg.
#[test]
fn test_atan2_quadrants_and_extreme_ratios() {
    let values = [
        0.0,
        f32::from_bits(1),
        f32::from_bits(2),
        f32::from_bits(0x007f_ffff),
        f32::MIN_POSITIVE,
        f32::from_bits(0x0080_0001),
        0.5,
        f32::from_bits(0x3f7f_ffff),
        1.0,
        f32::from_bits(0x3f80_0001),
        2.0,
        f32::MAX,
        f32::INFINITY,
        f32::NAN,
    ];
    let signed = values.into_iter().flat_map(|x| [x, -x]).collect::<Vec<_>>();
    let cases = || {
        signed
            .iter()
            .flat_map(|&y| signed.iter().map(move |&x| [y, x]))
    };
    common::test_bivariate_cases(metallic::atan2f, core_math::atan2f, cases());
    common::test_bivariate_cases(metallic::atan2pif, core_math::atan2pif, cases());

    let diagonal = (1..0x7f80_0000_u32).step_by(65_537).flat_map(|bits| {
        let x = f32::from_bits(bits);
        let y = f32::from_bits(bits + 1);
        [[y, x], [x, y], [y, -x], [-x, y], [-y, -x]]
    });
    common::test_bivariate_cases(metallic::atan2f, core_math::atan2f, diagonal.clone());
    common::test_bivariate_cases(metallic::atan2pif, core_math::atan2pif, diagonal);
}
