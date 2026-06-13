mod common;
use common::Identity as _;

#[test]
fn test_tgamma_exact() {
    // Exact integer factorials and the half-integer √π values.
    assert!(metallic::tgamma(1.0).eq(&1.0));
    assert!(metallic::tgamma(2.0).eq(&1.0));
    assert!(metallic::tgamma(3.0).eq(&2.0));
    assert!(metallic::tgamma(6.0).eq(&120.0));
    assert!(metallic::tgamma(0.5).eq(&1.772_453_850_905_516)); // √π
    assert!(metallic::tgamma(-0.5).eq(&-3.544_907_701_811_032)); // −2√π

    // Poles and signed infinities.
    assert!(metallic::tgamma(0.0).eq(&f64::INFINITY));
    assert!(metallic::tgamma(-0.0).eq(&f64::NEG_INFINITY));
    assert!(metallic::tgamma(-1.0).is_nan());
    assert!(metallic::tgamma(-100.0).is_nan());
    assert!(metallic::tgamma(f64::NAN).is_nan());

    // Overflow / underflow / infinities.
    assert!(metallic::tgamma(172.0).eq(&f64::INFINITY));
    assert!(metallic::tgamma(f64::INFINITY).eq(&f64::INFINITY));
    assert!(metallic::tgamma(f64::NEG_INFINITY).is_nan());
    assert!(metallic::tgamma(-200.5).eq(&0.0)); // even floor → +0
    assert!(metallic::tgamma(-200.3).eq(&-0.0)); // odd floor (−201) → −0
    assert!(metallic::tgamma(f64::from_bits(1)).eq(&f64::INFINITY)); // 1/z pole overflows
    assert!(metallic::tgamma(-f64::from_bits(1)).eq(&f64::NEG_INFINITY));
}

/// Regression guard over the frozen hard-to-round corpus.
///
/// The frozen answers come from MPFR, keeping this guard independent of
/// CORE-MATH (whose `f64::tgamma` only appeared in 1.1.1 — see
/// [`test_tgamma_worst_cases`] for that comparison).  `tests/cases/f64_tgamma.wc`
/// holds the inputs most likely to mis-round (results near an `f64` midpoint)
/// with their correctly-rounded results; verifying against frozen answers needs
/// no oracle, so this runs in the default `cargo test`.  Regenerate (and
/// re-verify on 800M fresh inputs) with
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
        let got = metallic::tgamma(z);
        (!got.is(&want)).then(|| println!("tgamma({z:e}) = {got:e} != {want:e} (correct)"))
    }));
}

/// Size of `tests/cases/f64_tgamma.wc` (kept in sync with the generator).
const CORPUS_LEN: usize = 1990;

/// Correct-rounding gate over CORE-MATH's official worst cases (the `--worst`
/// step).  `tests/cases/tgamma.wc` is CORE-MATH's full BaCSeL corpus (~12 MB),
/// committed to git but excluded from the published crate (`exclude` in
/// Cargo.toml); refresh it with `tools/sync-worst-cases.sh`.  RED until `tgamma`
/// is correctly rounded (it mis-rounds ~312 near-ties by 1 ulp; see issue #6);
/// [`test_tgamma_worst_faithful`] is the active ≤1-ulp floor.
#[test]
fn test_tgamma_worst_cases() {
    let cases: Vec<f64> = common::parse_case_file("tgamma.wc", common::parse_f64).collect();
    assert!(
        cases.is_empty() || cases.len() == 545_521,
        "corpus size changed; update this count"
    );
    common::test_univariate_cases(metallic::tgamma, core_math::tgamma, cases.into_iter());
}

/// Faithful-rounding floor (≤ 1 ulp) on that same corpus.
#[test]
fn test_tgamma_worst_faithful() {
    common::test_worst_faithful("tgamma", metallic::tgamma, core_math::tgamma, 1);
}

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
        let got = metallic::tgamma(z);
        let want = cr(z);
        (!got.is(&want)).then(|| println!("tgamma({z:e}) = {got:e} != {want:e} (correct)"))
    }));
}
