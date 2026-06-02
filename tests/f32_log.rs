#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

mod common;

/// Correctly-rounded reference via MPFR: `ln(x)/ln(base)` at 200 bits, rounded
/// once to `f32`.  Run with `cargo test --features mpfr`.
#[cfg(feature = "mpfr")]
fn correctly_rounded(x: f32, base: f32) -> f32 {
    use rug::Float;
    const PREC: u32 = 200;
    let lx = Float::with_val(PREC, x).ln();
    let lb = Float::with_val(PREC, base).ln();
    (lx / lb).to_f32()
}

#[cfg(feature = "mpfr")]
#[test]
fn test_log_correct_rounding() {
    use common::Identity as _;

    /// Finite positive normal `f32` from a hash: exponent in `[1, 254]`, hashed
    /// mantissa.
    fn posf(hash: u32) -> f32 {
        let exp = 1 + (hash >> 23) % 254;
        let mant = hash & 0x007F_FFFF;
        f32::from_bits((exp << 23) | mant)
    }

    // Wide (x, base) over the whole exponent range.
    let wide = (0..4_000_000u32).map(|i| {
        let x = posf(i.wrapping_mul(0x2545_F491));
        let base = posf(i.wrapping_mul(0x9E37_79B9) ^ 0xABCD);
        [x, base]
    });

    // base near 1 (large results — the hardest rounding), x ∈ [1, 2).
    let near1 = (0..4_000_000u32).map(|i| {
        let xb = 0x3F80_0000 | (i.wrapping_mul(0x9E37_79B9) >> 9);
        let bb = 0x3F80_0000 | (i.wrapping_mul(0xC2B2_AE3D) >> 9);
        [f32::from_bits(xb), f32::from_bits(bb)]
    });

    common::truncate_errors(wide.chain(near1).filter_map(|[x, base]| {
        let f = metallic::f32::log(x, base);
        let g = correctly_rounded(x, base);
        (!f.is(&g)).then(|| {
            let ulp = common::ulp_error_f32(f, g);
            println!("log({x:e}, {base:e}) = {f:e} != {g:e} ({ulp} ulp)")
        })
    }));
}
