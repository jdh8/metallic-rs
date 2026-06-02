mod common;
use metallic::f64 as metal;

#[test]
fn test_log_exact() {
    // `log2` of a power of two is exact, so these ratios are exact.
    assert!(metal::log(8.0, 2.0).eq(&3.0));
    assert!(metal::log(0.25, 2.0).eq(&-2.0));
    assert!(metal::log(2.0, 4.0).eq(&0.5));
    assert!(metal::log(125.0, 5.0).eq(&3.0));
    assert!(metal::log(1.0, 10.0).eq(&0.0));

    // Special inputs (∞/0/NaN/base 1) follow the f64 ratio of the log2 values.
    assert!(metal::log(f64::INFINITY, 2.0).is_infinite());
    assert!(metal::log(0.0, 2.0).eq(&f64::NEG_INFINITY));
    assert!(metal::log(-1.0, 2.0).is_nan());
    assert!(metal::log(8.0, 1.0).is_infinite());
    assert!(metal::log(1.0, 1.0).is_nan());
}

#[test]
fn test_log_vs_std() {
    // Sanity vs `std`, which is only faithfully rounded, so allow 2 ulps (correct
    // rounding is verified bit-exact by `test_log_correct_rounding` under
    // `--features mpfr`).  Skip non-normal and near-1 inputs (result ≈ 0).
    for i in (0..f64::INFINITY.to_bits()).step_by((1 << 46) + 1) {
        let x = f64::from_bits(i);
        if !x.is_normal() {
            continue;
        }
        for base in [2.0_f64, 3.0, 7.5, 10.0] {
            let got = metal::log(x, base);
            let want = x.log(base);
            if want.abs() < 1e-6 {
                continue;
            }
            let ulps = (got.to_bits() as i64 - want.to_bits() as i64).abs();
            assert!(
                ulps <= 2,
                "log({x:e}, {base}) = {got:e} vs std {want:e} ({ulps} ulps)"
            );
        }
    }
}

/// Correctly-rounded reference via MPFR: `ln(x)/ln(base)` at 200 bits, rounded
/// once to f64.  Run with `cargo test --features mpfr`.
#[cfg(feature = "mpfr")]
fn correctly_rounded(x: f64, base: f64) -> f64 {
    use rug::Float;
    const PREC: u32 = 200;
    let lx = Float::with_val(PREC, x).ln();
    let lb = Float::with_val(PREC, base).ln();
    (lx / lb).to_f64()
}

#[cfg(feature = "mpfr")]
#[test]
fn test_log_correct_rounding() {
    /// Finite positive normal f64 from a hash: exponent in [1, 2046], hashed
    /// mantissa.
    fn posf(hash: u64) -> f64 {
        let exp = 1 + (hash >> 52) % 2046;
        let mant = hash & 0x000F_FFFF_FFFF_FFFF;
        f64::from_bits((exp << 52) | mant)
    }

    // Wide (x, base) over the whole exponent range.
    let wide = (0..2_000_000u64).map(|i| {
        let x = posf(i.wrapping_mul(0x2545_F491_4F6C_DD1D));
        let base = posf(i.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xABCD);
        [x, base]
    });

    // base near 1 (large results — the hardest rounding), x ∈ [1, 2).
    let near1 = (0..2_000_000u64).map(|i| {
        let xb = 0x3FF0_0000_0000_0000 | (i.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 12);
        let bb = 0x3FF0_0000_0000_0000 | (i.wrapping_mul(0xC2B2_AE3D_27D4_EB4F) >> 14);
        [f64::from_bits(xb), f64::from_bits(bb)]
    });

    common::test_bivariate_cases(metal::log, correctly_rounded, wide.chain(near1));
}
