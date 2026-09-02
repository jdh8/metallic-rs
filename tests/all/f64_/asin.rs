use crate::common;

#[test]
fn test_asin() {
    let dense = (0..=4_000_000).map(|i| metallic::fma(f64::from(i), 2.0 / 4_000_000.0, -1.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    // CORE-MATH's published hard-to-round asin arguments (and their negatives, by
    // symmetry), which the dense/bit sweeps do not hit exactly: the original two
    // plus the exceptional-case table of CORE-MATH 1.0.2's rewritten asin.c.
    let hard = [
        "0x1.fffffffffffffp-1",
        "0x1.fffffffffffffp-7",
        "0x1.7137449123ef6p-26",
        "0x1.d12ed0af1a27ep-26",
        "0x1.51c4b960778f5p-23",
        "0x1.3cfc2a006a414p-22",
        "0x1.cbaa95dadb559p-22",
        "0x1.acd69f89ad8f1p-20",
        "0x1.2bffffffc233bp-16",
        "0x1.ff0f3022b2e9dp-16",
        "0x1.3217783d70d1dp-14",
        "0x1.c373ff4aad79bp-14",
        "0x1.b3f28593cad2fp-9",
        "0x1.e17b3f6bb5e6ep-7",
        "0x1.41d60a76a82edp-6",
        "0x1.921c0a0486537p-6",
        "0x1.9c360a8dd681ap-6",
        "0x1.d6315f7ee7e01p-6",
        "0x1.ea6fdc56fc61ap-6",
        "0x1.2749dc4d19c6dp-5",
        "0x1.69768dc89bbp-5",
        "0x1.a4816b2066707p-5",
        "0x1.d77b117f230d6p-5",
        "0x1.fc7a07b2549aap-5",
        "0x1.2df0542154f1bp-4",
        "0x1.51cf5db1b1956p-4",
        "0x1.d0ef799001ba9p-3",
        "0x1.4a8e1a96e38e3p-2",
        "0x1.ceee68154d1c9p-2",
        "0x1.da4e0e6c717a5p-2",
        "0x1.e9950730c4696p-2",
    ]
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

#[test]
fn test_asin_worst_cases() {
    common::test_worst_univariate("asin", metallic::asin, core_math::asin);
}

/// Independent confirmation of correct rounding against MPFR.  Run with
/// `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_asin_vs_mpfr() {
    let cr = |x: f64| rug::Float::with_val(200, x).asin().to_f64();
    common::mpfr_sweep_univariate(
        metallic::asin,
        cr,
        |i| common::uniform(common::mix64(i), -1.0, 1.0),
        2_000_000,
    );
}
