mod common;
use common::Identity as _;
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
    // rounding is verified bit-exact by `test_log_corpus`).  Skip non-normal and
    // near-1 inputs (result ≈ 0).
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

/// Regression guard over the frozen hard-to-round corpus.
///
/// `tests/cases/f64_log.wc` holds the inputs `metallic::f64::log` is most likely
/// to mis-round (results near an `f64` midpoint) together with their
/// correctly-rounded results, computed once via MPFR.  Verifying against those
/// frozen answers needs no oracle, so this runs in the default `cargo test` (and
/// in CI).  Regenerate it with
/// `cargo run --release --features mpfr --example gen_f64_log_cases`; the
/// generator also scans billions of fresh inputs and asserts none mis-round.
#[test]
fn test_log_corpus() {
    let cases: Vec<[f64; 3]> =
        common::parse_case_file("f64_log.wc", common::parse_f64_triple).collect();
    assert_eq!(cases.len(), 2868, "corpus size changed; update this count");

    common::truncate_errors(cases.into_iter().filter_map(|[x, base, want]| {
        let got = metal::log(x, base);
        (!got.is(&want)).then(|| println!("log({x:e}, {base:e}) = {got:e} != {want:e} (correct)"))
    }));
}
