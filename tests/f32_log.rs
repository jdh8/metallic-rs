#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

mod common;
use common::Identity as _;

/// Regression guard over the frozen hard-to-round corpus.
///
/// `tests/cases/f32_log.wc` holds the inputs `metallic::f32::log` is most likely
/// to mis-round (results near an `f32` midpoint) together with their
/// correctly-rounded results, computed once via MPFR.  Verifying against those
/// frozen answers needs no oracle, so this runs in the default `cargo test` (and
/// in CI).  Regenerate it with
/// `cargo run --release --features mpfr --example gen_f32_log_cases`; the
/// generator also scans hundreds of millions of fresh inputs and asserts none
/// mis-round.
#[test]
fn test_log_corpus() {
    let cases: Vec<[f32; 3]> =
        common::parse_case_file("f32_log.wc", common::parse_f32_triple).collect();
    assert_eq!(cases.len(), 1120, "corpus size changed; update this count");

    common::truncate_errors(cases.into_iter().filter_map(|[x, base, want]| {
        let got = metallic::f32::log(x, base);
        (!got.is(&want)).then(|| println!("log({x:e}, {base:e}) = {got:e} != {want:e} (correct)"))
    }));
}
