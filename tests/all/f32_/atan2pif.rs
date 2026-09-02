use crate::common;

#[test]
fn test_atan2pif() {
    // A dense (y, x) grid over [-5, 5]² plus hashed wide-magnitude pairs.
    let grid = (0..3000u64).flat_map(|i| {
        (0..3000u64).map(move |j| {
            #[allow(clippy::cast_possible_truncation)]
            let y = metallic::fmaf(i as f32, 10.0 / 3000.0, -5.0);
            #[allow(clippy::cast_possible_truncation)]
            let x = metallic::fmaf(j as f32, 10.0 / 3000.0, -5.0);
            [y, x]
        })
    });
    let wide = (0..4_000_000u64).map(|i| {
        let h = common::mix64(i);
        #[allow(clippy::cast_possible_truncation)]
        [f32::from_bits(h as u32), f32::from_bits((h >> 32) as u32)]
    });
    common::test_bivariate_cases(metallic::atan2pif, core_math::atan2pif, grid.chain(wide));
}

/// Strict correct-rounding gate on CORE-MATH's hard-to-round corpus.
#[test]
fn test_atan2pif_worst_cases() {
    common::test_bivariate_cases(
        metallic::atan2pif,
        core_math::atan2pif,
        common::parse_case_file("atan2pif.wc", common::parse_f32_pair),
    );
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_atan2pif_worst_faithful() {
    common::truncate_errors(
        common::parse_case_file("atan2pif.wc", common::parse_f32_pair).filter_map(|[y, x]| {
            let (a, b) = (metallic::atan2pif(y, x), core_math::atan2pif(y, x));
            let error = common::ulp_error_f32(a, b);
            (error > 1).then(|| println!("{y:e}, {x:e}: {a:e} != {b:e} ({error} ulp)"))
        }),
    );
}
