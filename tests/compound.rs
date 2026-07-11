mod common;
use common::Identity as _;

/// Special values (C23 F.10.4.1) and exact short dyadics.
#[test]
fn test_compound_exact() {
    // compound(±0, y) = 1 for every y (quiet NaN included); sNaN → NaN.
    assert!(metallic::compound(0.0, 3.0).eq(&1.0));
    assert!(metallic::compound(-0.0, -2.0).eq(&1.0));
    assert!(metallic::compound(0.0, f64::INFINITY).eq(&1.0));
    assert!(metallic::compound(0.0, f64::NAN).eq(&1.0));

    // compound(x, ±0) = 1 for x ≥ −1 (quiet NaN included); x < −1 or sNaN → NaN.
    assert!(metallic::compound(5.0, 0.0).eq(&1.0));
    assert!(metallic::compound(f64::INFINITY, 0.0).eq(&1.0));
    assert!(metallic::compound(f64::NAN, 0.0).eq(&1.0));
    assert!(metallic::compound(-2.0, 0.0).is_nan()); // x < −1 domain error

    // x = −1: +0 for y > 0, +∞ for y < 0.
    assert!(metallic::compound(-1.0, 2.0).eq(&0.0));
    assert!(metallic::compound(-1.0, -2.0).eq(&f64::INFINITY));

    // x < −1 (incl. −∞) is a domain error.
    assert!(metallic::compound(-2.0, 3.0).is_nan());
    assert!(metallic::compound(f64::NEG_INFINITY, 2.0).is_nan());

    // Infinite exponent: (1+x) vs 1 decides growth or decay.
    assert!(metallic::compound(0.5, f64::INFINITY).eq(&f64::INFINITY));
    assert!(metallic::compound(-0.5, f64::INFINITY).eq(&0.0));
    assert!(metallic::compound(0.5, f64::NEG_INFINITY).eq(&0.0));

    // x = +∞.
    assert!(metallic::compound(f64::INFINITY, 2.0).eq(&f64::INFINITY));
    assert!(metallic::compound(f64::INFINITY, -2.0).eq(&0.0));

    // NaN propagation.
    assert!(metallic::compound(f64::NAN, 2.0).is_nan());
    assert!(metallic::compound(0.5, f64::NAN).is_nan());

    // y = 1 returns the correctly-rounded 1 + x.
    assert!(metallic::compound(3.0, 1.0).eq(&4.0));

    // Exact integer / dyadic powers.
    assert!(metallic::compound(1.0, 2.0).eq(&4.0)); // 2²
    assert!(metallic::compound(3.0, 2.0).eq(&16.0)); // 4²
    assert!(metallic::compound(0.5, 2.0).eq(&2.25)); // 1.5²
    assert!(metallic::compound(7.0, 3.0).eq(&512.0)); // 8³
    assert!(metallic::compound(3.0, 0.5).eq(&2.0)); // √4
}

/// Wherever `1 + x` is exactly representable, `(1+x)ʸ` is the same real value as
/// `pow(1.0 + x, y)`; both are correctly rounded, so they must agree bit-for-bit.
/// This reuses the (already correctly-rounded) `pow` as a free oracle over the
/// bulk of the domain — no MPFR — leaving only the small-|x| region (where
/// `1 + x` loses bits) to the corpus and the MPFR sweep.
#[test]
fn test_compound_vs_pow() {
    common::truncate_errors((0..30_000_000u64).filter_map(|i| {
        let h = common::mix64(i);
        let (x, y) = if i & 1 == 0 {
            // Base near 1: s ∈ [0.5, 2), x = s − 1 (exact by Sterbenz); the hard
            // regime, with large |y| to maximise the ×y amplification.
            let s = f64::from_bits(0x3FE0_0000_0000_0000 | (h >> 12)); // [0.5, 1)
            let s = if h & 1 == 0 { s } else { s * 2.0 }; // [0.5, 2)
            (
                s - 1.0,
                common::uniform(common::mix64(i ^ 0xABCD), -1100.0, 1100.0),
            )
        } else {
            // Integer x (so 1 + x is exact) across a wide range, moderate y.
            let x = ((h >> 20) % (1 << 40)) as f64 * if h & 1 == 0 { 1.0 } else { -1.0 };
            (x, common::uniform(common::mix64(i ^ 0x1234), -40.0, 40.0))
        };

        // Guard: skip x = 0 and confirm 1 + x is genuinely exact.
        let s = 1.0 + x;
        let c = if x <= 1.0 {
            x - (s - 1.0)
        } else {
            1.0 - (s - x)
        };
        if x == 0.0 || !(x > -1.0) || c != 0.0 {
            return None;
        }

        let got = metallic::compound(x, y);
        let want = metallic::pow(s, y);
        (!got.is(&want))
            .then(|| println!("compound({x:e}, {y:e}) = {got:e} != pow({s:e}, {y:e}) = {want:e}"))
    }));
}

/// Regression guard over the frozen hard-to-round corpus.
///
/// `compound` has no correctly-rounded `f64` reference in the `core-math` crate
/// (only `compoundf`), so MPFR is the oracle.  `tests/cases/f64_compound.wc`
/// holds the inputs most likely to mis-round — small `|x|`, where `1 + x` is
/// inexact and `pow` cannot cross-check — with their correctly-rounded results;
/// verifying against frozen answers needs no oracle, so this runs in the default
/// `cargo test`.  Regenerate with
/// `cargo run --release --features mpfr --example gen_f64_compound_cases`.
#[test]
fn test_compound_corpus() {
    let cases: Vec<[f64; 3]> =
        common::parse_case_file("f64_compound.wc", common::parse_f64_triple).collect();
    assert_eq!(
        cases.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );

    common::truncate_errors(cases.into_iter().filter_map(|[x, y, want]| {
        let got = metallic::compound(x, y);
        (!got.is(&want)).then(|| println!("compound({x:e}, {y:e}) = {got:e} != {want:e} (correct)"))
    }));
}

/// Size of `tests/cases/f64_compound.wc` (kept in sync with the generator).
const CORPUS_LEN: usize = 2925;

/// Broad correct-rounding sweep against MPFR (gated behind `mpfr`, like the
/// corpus generator).  `(1 + x)^y` at 256 bits — the 256-bit sum `1 + x` is
/// exact for every finite f64 `x`.  Concentrated on small `|x|` and large `|y|`,
/// the region `test_compound_vs_pow` cannot reach.  Run with
/// `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_compound_vs_mpfr() {
    use rug::Float;
    use rug::ops::Pow;

    let cr = |x: f64, y: f64| {
        (Float::with_val(256, x) + 1_u32)
            .pow(Float::with_val(256, y))
            .to_f64()
    };

    common::truncate_errors((0..8_000_000u64).filter_map(|i| {
        let h = common::mix64(i);
        let (x, y) = match i % 4 {
            // Small |x| (base near 1), large |y|: 1 + x inexact, max amplification.
            0 => {
                let u = (h >> 11) as f64 / (1u64 << 53) as f64;
                let s = metallic::fma(u, 2.0, -1.0); // [−1, 1)
                let k = 1 + (common::mix64(i ^ 0x5555) % 60) as i64;
                let x = s * f64::from_bits(((0x3FF - k) as u64) << 52); // × 2⁻ᵏ
                (
                    x,
                    common::uniform(common::mix64(i ^ 0xABCD), -1100.0, 1100.0),
                )
            }
            // Medium |x| in (−1, 1), full-range y.
            1 => {
                let x = common::uniform(h, -0.999, 1.0);
                (x, common::uniform(common::mix64(i ^ 0x1234), -700.0, 700.0))
            }
            // Wide x, moderate y.
            2 => {
                let x = common::uniform(h, -0.5, 1e6);
                (x, common::uniform(common::mix64(i ^ 0x9999), -40.0, 40.0))
            }
            // Huge x (1 + x collapses to x; the correction must capture 1/x), with
            // a target result exponent so the value stays in range.
            _ => {
                let x = f64::from_bits(0x4000_0000_0000_0000 | (h >> 12)); // ~[2, 2¹⁰²⁴)
                let e = common::uniform(common::mix64(i ^ 0x7777), -1060.0, 1020.0);
                (x, e / (std::f64::consts::LOG2_E * x.ln_1p()))
            }
        };
        if !(x.is_finite() && x > -1.0 && x != 0.0) {
            return None;
        }
        let got = metallic::compound(x, y);
        let want = cr(x, y);
        (!got.is(&want)).then(|| println!("compound({x:e}, {y:e}) = {got:e} != {want:e} (correct)"))
    }));
}
