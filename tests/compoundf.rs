mod common;

#[test]
fn test_parser() {
    assert!(common::parse_case_file("compoundf.wc", common::parse_f32_pair).count() == 297_535);
}

#[test]
fn test_compoundf() {
    common::test_bivariate_cases(
        metallic::compoundf,
        core_math::compoundf,
        common::parse_case_file("compoundf.wc", common::parse_f32_pair),
    );
}

/// Independent confirmation against MPFR: `(1 + x)^y` evaluated at 200 bits
/// (the 200-bit sum `1 + x` is exact for every finite f32 `x`), sampled
/// value-uniform over the kernel band and all sign quadrants.  Run with
/// `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_compoundf_vs_mpfr() {
    use rug::ops::Pow;

    let mut fails = 0;
    for i in 0..2_000_000_u64 {
        let h = common::mix64(i);
        #[allow(clippy::cast_possible_truncation)]
        let x = f32::from_bits((h >> 32) as u32);
        #[allow(clippy::cast_possible_truncation)]
        let y = f32::from_bits(h as u32);
        if !x.is_finite() || !y.is_finite() || x < -1.0 {
            continue;
        }
        let want = (rug::Float::with_val(200, x) + 1_u32)
            .pow(rug::Float::with_val(200, y))
            .to_f32();
        let got = metallic::compoundf(x, y);
        if got.to_bits() != want.to_bits() && !(got.is_nan() && want.is_nan()) {
            println!("{x:e}, {y:e}: {got:e} != {want:e} (correct)");
            fails += 1;
            assert!(fails < 50, "too many mismatches");
        }
    }
    assert!(fails == 0, "There are {fails} mismatches");
}
