mod common;
use common::Identity as _;
use metallic::f64 as metal;

#[test]
fn test_lgamma_exact() {
    // Exact zeros at Γ(1) = Γ(2) = 1 and the √π / 2√π half-integer values.
    assert!(metal::lgamma(1.0).eq(&0.0));
    assert!(metal::lgamma(2.0).eq(&0.0));
    assert!(metal::lgamma(0.5).eq(&5.723_649_429_247_001e-1)); // ln √π
    assert!(metal::lgamma(-0.5).eq(&1.265_512_123_484_645_4)); // ln 2√π
    assert!(metal::lgamma(3.0).eq(&core::f64::consts::LN_2)); // ln 2! = ln 2

    // Poles and infinities: non-positive integers → +∞.
    assert!(metal::lgamma(0.0).eq(&f64::INFINITY));
    assert!(metal::lgamma(-0.0).eq(&f64::INFINITY));
    assert!(metal::lgamma(-1.0).eq(&f64::INFINITY));
    assert!(metal::lgamma(-100.0).eq(&f64::INFINITY));
    assert!(metal::lgamma(-1e6).eq(&f64::INFINITY)); // large integer → pole
    assert!(metal::lgamma(f64::INFINITY).eq(&f64::INFINITY));
    assert!(metal::lgamma(f64::NEG_INFINITY).eq(&f64::INFINITY)); // even integer pole
    assert!(metal::lgamma(f64::NAN).is_nan());

    // Grows without overflow until ~1.4e306; remains finite at 1e300.
    assert!(metal::lgamma(1e300).is_finite());
}

/// Regression guard over the frozen hard-to-round corpus.
///
/// `core-math` has no `lgamma` for `f64` (only `lgammaf`), so the correctly-rounded
/// oracle is MPFR.  `tests/cases/f64_lgamma.wc` holds the inputs most likely to
/// mis-round; verifying against those frozen answers needs no oracle, so this runs
/// in the default `cargo test`.  Regenerate (and re-verify on 800M fresh inputs)
/// with `cargo run --release --features mpfr --example gen_f64_lgamma_cases`.
#[test]
fn test_lgamma_corpus() {
    let cases: Vec<[f64; 2]> =
        common::parse_case_file("f64_lgamma.wc", common::parse_f64_pair).collect();
    assert_eq!(
        cases.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );

    common::truncate_errors(cases.into_iter().filter_map(|[z, want]| {
        let got = metal::lgamma(z);
        (!got.is(&want)).then(|| println!("lgamma({z:e}) = {got:e} != {want:e} (correct)"))
    }));
}

/// Size of `tests/cases/f64_lgamma.wc` (kept in sync with the generator).
const CORPUS_LEN: usize = 585;

/// Broad correct-rounding sweep against MPFR (gated behind `mpfr`).  Run with
/// `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_lgamma_vs_mpfr() {
    use rug::Float;

    let cr = |z: f64| Float::with_val(220, z).ln_abs_gamma().0.to_f64();
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    common::truncate_errors((0..4_000_000u64).filter_map(|_| {
        // representation-uniform `z` over (−1e6, 1e6), skipping the poles.
        let e = 1023 - (next() % 50); // exponent in [974, 1022] → |z| ∈ [2⁻⁵⁰, 2)
        let scale = (next() % 19) as f64; // up to ~×2¹⁹
        let mag = f64::from_bits((e << 52) | (next() >> 12)) * (scale + 1.0);
        let z = if next() & 1 == 0 { mag } else { -mag };
        if z < 0.5 && (z - z.round()).abs() < 1.0 / 1024.0 {
            return None;
        }
        let got = metal::lgamma(z);
        let want = cr(z);
        (!got.is(&want)).then(|| println!("lgamma({z:e}) = {got:e} != {want:e} (correct)"))
    }));
}
