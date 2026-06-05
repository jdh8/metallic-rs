mod common;
use common::Identity as _;
use metallic::f64 as metal;

#[test]
fn test_tgamma_exact() {
    // Exact integer factorials and the half-integer √π values.
    assert!(metal::tgamma(1.0).eq(&1.0));
    assert!(metal::tgamma(2.0).eq(&1.0));
    assert!(metal::tgamma(3.0).eq(&2.0));
    assert!(metal::tgamma(6.0).eq(&120.0));
    assert!(metal::tgamma(0.5).eq(&1.772_453_850_905_516)); // √π
    assert!(metal::tgamma(-0.5).eq(&-3.544_907_701_811_032)); // −2√π

    // Poles and signed infinities.
    assert!(metal::tgamma(0.0).eq(&f64::INFINITY));
    assert!(metal::tgamma(-0.0).eq(&f64::NEG_INFINITY));
    assert!(metal::tgamma(-1.0).is_nan());
    assert!(metal::tgamma(-100.0).is_nan());
    assert!(metal::tgamma(f64::NAN).is_nan());

    // Overflow / underflow / infinities.
    assert!(metal::tgamma(172.0).eq(&f64::INFINITY));
    assert!(metal::tgamma(f64::INFINITY).eq(&f64::INFINITY));
    assert!(metal::tgamma(f64::NEG_INFINITY).is_nan());
    assert!(metal::tgamma(-200.5).eq(&0.0)); // even floor → +0
    assert!(metal::tgamma(-200.3).eq(&-0.0)); // odd floor (−201) → −0
    assert!(metal::tgamma(f64::from_bits(1)).eq(&f64::INFINITY)); // 1/z pole overflows
    assert!(metal::tgamma(-f64::from_bits(1)).eq(&f64::NEG_INFINITY));
}

/// Regression guard over the frozen hard-to-round corpus.
///
/// `core-math` has no `tgamma` for `f64` (only `tgammaf`), so the correctly-rounded
/// oracle is MPFR.  `tests/cases/f64_tgamma.wc` holds the inputs most likely to
/// mis-round (results near an `f64` midpoint) with their correctly-rounded results;
/// verifying against those frozen answers needs no oracle, so this runs in the
/// default `cargo test`.  Regenerate (and re-verify on 800M fresh inputs) with
/// `cargo run --release --features mpfr --example gen_f64_tgamma_cases`.
#[test]
fn test_tgamma_corpus() {
    let cases: Vec<[f64; 2]> =
        common::parse_case_file("f64_tgamma.wc", common::parse_f64_pair).collect();
    assert_eq!(
        cases.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );

    common::truncate_errors(cases.into_iter().filter_map(|[z, want]| {
        let got = metal::tgamma(z);
        (!got.is(&want)).then(|| println!("tgamma({z:e}) = {got:e} != {want:e} (correct)"))
    }));
}

/// Size of `tests/cases/f64_tgamma.wc` (kept in sync with the generator).
const CORPUS_LEN: usize = 1990;

/// Broad correct-rounding sweep against MPFR (gated behind `mpfr`, like the
/// corpus generator, so the default test stays dependency-light).  Run with
/// `cargo test --release --features mpfr`.
#[cfg(feature = "mpfr")]
#[test]
fn test_tgamma_vs_mpfr() {
    use rug::Float;

    let cr = |z: f64| Float::with_val(220, z).gamma().to_f64();
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    common::truncate_errors((0..4_000_000u64).filter_map(|_| {
        // representation-uniform `z` over (−179.5, 171.6), skipping the poles.
        let e = 1022 - (next() % 40); // exponent in [983, 1022] → |z| ∈ [2⁻⁴⁰, 2)
        let scale = (next() % 8) as f64 + 1.0; // up to ~×8 → |z| up to ~16
        let mag = f64::from_bits((e << 52) | (next() >> 12)) * scale;
        let z = if next() & 1 == 0 { mag } else { -mag };
        if !(-179.5..171.6).contains(&z) || (z < 0.5 && (z - z.round()).abs() < 1.0 / 1024.0) {
            return None;
        }
        let got = metal::tgamma(z);
        let want = cr(z);
        (!got.is(&want)).then(|| println!("tgamma({z:e}) = {got:e} != {want:e} (correct)"))
    }));
}
