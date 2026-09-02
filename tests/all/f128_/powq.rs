use crate::common;
use crate::common128;

use common::Identity as _;

// CORE-MATH has not shipped `powq` yet, so the strict gate replays a
// home-grown corpus that carries its own MPFR answers
// (`examples/gen_f128_pow_cases.rs`) and runs under plain `--features f128`.
// The parser count keeps the corpus from rotting unnoticed, the f64 oracle
// covers every binade with no MPFR, and the MPFR sweeps stay the independent
// cross-check.

/// Size of `tests/cases/powq.wc` (kept in sync with the generator).
const CORPUS_LEN: usize = 37_010;

const SAMPLE_COUNT: u64 = 200_000;

const FRACTION: u128 = (1 << 112) - 1;

/// A positive `x` with the exponent uniform over every binade, subnormals
/// included, and a `y` whose exponent keeps `|y·log2 x|` under 2^14 most of
/// the time — the whole finite range, both signs of `y`.
fn general(i: u64) -> [f128; 2] {
    let xb = common128::mix128(2 * i);
    let yb = common128::mix128(2 * i + 1);
    let ex = (xb >> 112) % 0x7fff;
    let x = f128::from_bits(ex << 112 | (xb & FRACTION));
    let scale = (ex as i64 - 16383).unsigned_abs().max(1).ilog2() as i64;
    let ey = 16383 - 120 + ((yb >> 112) % (135 - scale) as u128) as i64;

    [
        x,
        f128::from_bits(yb & 1 << 127 | (ey as u128) << 112 | (yb & FRACTION)),
    ]
}

/// `x = 1 ± t` for `t` log-uniform over `[2^-113, 2^-1)` under a `y` large
/// enough for `|y·log2 x|` to reach 2^13: the accurate leg's floating
/// logarithm, and the fast leg's `|y|` hand-over.
fn near_one(i: u64) -> [f128; 2] {
    let xb = common128::mix128(2 * i);
    let yb = common128::mix128(2 * i + 1);
    let k = (xb >> 112 & 0x7fff) % 113;
    let t = f128::from_bits((16383 - 113 + k) << 112 | (xb & FRACTION));
    let x = if xb >> 127 == 0 { 1.0 + t } else { 1.0 - t };
    let ey = 16383 + k as i64 - 120 + ((yb >> 112) % 134) as i64;

    [
        x,
        f128::from_bits(yb & 1 << 127 | (ey as u128) << 112 | (yb & FRACTION)),
    ]
}

/// Integer exponents in `[−300, 300]` on a base of either sign in `[1, 4)`:
/// the sign fold and the exact family's neighbours.
fn integers(i: u64) -> [f128; 2] {
    let xb = common128::mix128(2 * i);
    let n = (common::mix64(i) % 601) as f128 - 300.0;

    [
        f128::from_bits(xb & 1 << 127 | (16383 + (xb >> 120 & 1)) << 112 | (xb & FRACTION)),
        n,
    ]
}

/// The benchmark's band: `|x| ∈ [2^-16, 2^17)`, `|y| ∈ [2^-16, 2^9)`.
fn band(i: u64) -> [f128; 2] {
    let xb = common128::mix128(2 * i);
    let yb = common128::mix128(2 * i + 1);
    let ex = 16383 - 16 + (xb >> 112) % 33;
    let ey = 16383 - 16 + (yb >> 112) % 25;

    [
        f128::from_bits(ex << 112 | (xb & FRACTION)),
        f128::from_bits(yb & 1 << 127 | ey << 112 | (yb & FRACTION)),
    ]
}

#[test]
fn test_powq_corpus() {
    let cases: Vec<[f128; 3]> =
        common::parse_case_file("powq.wc", common128::parse_f128_triple).collect();
    assert_eq!(
        cases.len(),
        CORPUS_LEN,
        "corpus size changed; update this count"
    );

    common::truncate_errors(cases.into_iter().filter_map(|[x, y, want]| {
        let got = metallic::powq(x, y);
        (!got.is(&want)).then(|| println!("powq({x:?}, {y:?}) = {got:?} != {want:?} (correct)"))
    }));
}

/// Every `f64` is a binary128 value, and its correctly rounded `f64` power
/// can differ from the binary128 one rounded down to `f64` by at most one ulp
/// (the double rounding).  With CORE-MATH's `pow` as the oracle this checks
/// every binade of the base with no MPFR at all — positive bases under a
/// moderate exponent, and negative bases under an integer one.
#[test]
fn test_powq_vs_f64() {
    common::truncate_errors((0..SAMPLE_COUNT).filter_map(|i| {
        let h = common::mix64(2 * i);
        let g = common::mix64(2 * i + 1);
        let (x, y) = if i % 4 == 0 {
            let x = f64::from_bits(1 << 63 | (0x3fe + (h >> 62)) << 52 | (h & (1 << 52) - 1));
            (x, (g % 601) as f64 - 300.0)
        } else {
            let x = f64::from_bits(((h >> 52) % 0x7ff) << 52 | (h & (1 << 52) - 1));
            let ey = 0x3ff - 30 + (g >> 52) % 40;
            (
                x,
                f64::from_bits(g & 1 << 63 | ey << 52 | (g & (1 << 52) - 1)),
            )
        };
        let got = metallic::powq(x as f128, y as f128) as f64;
        let want = core_math::pow(x, y);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("powq({x:e}, {y:e}) = {got:e} != {want:e}"))
    }));
}

#[test]
fn test_powq_special() {
    let inf = f128::INFINITY;
    assert!(metallic::powq(f128::NAN, 0.0).is(&1.0));
    assert!(metallic::powq(f128::NAN, -0.0).is(&1.0));
    assert!(metallic::powq(1.0, f128::NAN).is(&1.0));
    assert!(metallic::powq(1.0, inf).is(&1.0));
    assert!(metallic::powq(-1.0, inf).is(&1.0));
    assert!(metallic::powq(-1.0, -inf).is(&1.0));
    assert!(metallic::powq(-1.0, f128::NAN).is_nan());
    assert!(metallic::powq(f128::NAN, 1.0).is_nan());
    assert!(metallic::powq(2.0, f128::NAN).is_nan());
    assert!(metallic::powq(-2.0, 0.5).is_nan());
    assert!(metallic::powq(-f128::MIN_POSITIVE, 1.5).is_nan());
    assert!(metallic::powq(-inf, 0.5).is(&inf));
    assert!(metallic::powq(-inf, 3.0).is(&-inf));
    assert!(metallic::powq(-inf, -3.0).is(&-0.0));
    assert!(metallic::powq(-inf, -2.0).is(&0.0));
    assert!(metallic::powq(inf, -0.5).is(&0.0));
    assert!(metallic::powq(inf, 0.5).is(&inf));
    assert!(metallic::powq(0.5, inf).is(&0.0));
    assert!(metallic::powq(0.5, -inf).is(&inf));
    assert!(metallic::powq(1.5, inf).is(&inf));
    assert!(metallic::powq(1.5, -inf).is(&0.0));
    assert!(metallic::powq(0.0, inf).is(&0.0));
    assert!(metallic::powq(-0.0, -inf).is(&inf));
    assert!(metallic::powq(0.0, 3.0).is(&0.0));
    assert!(metallic::powq(-0.0, 3.0).is(&-0.0));
    assert!(metallic::powq(-0.0, 2.0).is(&0.0));
    assert!(metallic::powq(-0.0, 0.5).is(&0.0));
    assert!(metallic::powq(0.0, -3.0).is(&inf));
    assert!(metallic::powq(-0.0, -3.0).is(&-inf));
    assert!(metallic::powq(-0.0, -0.5).is(&inf));
    assert!(metallic::powq(-2.0, 3.0).is(&-8.0));
    assert!(metallic::powq(-2.0, -2.0).is(&0.25));
    assert!(metallic::powq(-1.0, f128::MAX).is(&1.0));
    assert!(metallic::powq(-1.0, -f128::MAX).is(&1.0));
    assert!(metallic::powq(f128::MAX, 2.0).is(&inf));
    assert!(metallic::powq(f128::MAX, -2.0).is(&0.0));
    assert!(metallic::powq(f128::MIN_POSITIVE, 2.0).is(&0.0));
    assert!(metallic::powq(f128::MIN_POSITIVE, -2.0).is(&inf));
}

/// The exact family: powers of two down to the subnormal midpoint, integer
/// powers of an odd base up to the 114-bit midpoint, and their roots.
#[test]
fn test_powq_exact() {
    assert!(metallic::powq(2.0, 10.0).is(&1024.0));
    assert!(metallic::powq(2.0, -10.0).is(&(1.0 / 1024.0)));
    assert!(metallic::powq(2.0, 16383.0).is(&f128::from_bits(0x7ffe << 112)));
    assert!(metallic::powq(2.0, 16384.0).is(&f128::INFINITY));
    assert!(metallic::powq(2.0, -16494.0).is(&f128::from_bits(1)));
    assert!(metallic::powq(2.0, -16495.0).is(&0.0));
    assert!(metallic::powq(4.0, -8247.5).is(&0.0));
    assert!(metallic::powq(4.0, -8247.0).is(&f128::from_bits(1)));
    assert!(metallic::powq(f128::from_bits(1), 0.5).is(&f128::from_bits(8136 << 112)));
    assert!(metallic::powq(9.0, 0.5).is(&3.0));
    assert!(metallic::powq(81.0, 0.75).is(&27.0));
    assert!(metallic::powq(3.0, 71.0).is(&(3_u128.pow(71) as f128)));
    assert!(metallic::powq(9.0, 35.5).is(&(3_u128.pow(71) as f128)));
    assert!(metallic::powq(-3.0, 71.0).is(&-(3_u128.pow(71) as f128)));

    // `5^49` has 114 bits, an exact midpoint: ties to even.
    let k = 5_u128.pow(49);
    let even = if (k >> 1) & 1 == 0 { k - 1 } else { k + 1 } as f128;
    assert!(metallic::powq(5.0, 49.0).is(&even));
    assert!(metallic::powq(25.0, 24.5).is(&even));
    assert!(metallic::powq(625.0, 12.25).is(&even));
    assert!(metallic::powq(-5.0, 49.0).is(&-even));

    // `3^64` is a perfect 64th power base: `x^(N/64)` is exact for odd `N`.
    let base = 3_u128.pow(64) as f128;
    assert!(metallic::powq(base, 1.0 / 64.0).is(&3.0));
    assert!(metallic::powq(base, 65.0 / 64.0).is(&(3_u128.pow(65) as f128)));
}

#[cfg(feature = "mpfr")]
#[test]
fn test_powq_vs_mpfr() {
    use rug::float::Round::Nearest;
    use rug::ops::PowAssignRound as _;

    for sampler in [general, near_one, integers, band] {
        common128::mpfr_sweep_bivariate_f128(
            metallic::powq,
            |x, y| metallic::f128_mpfr::cr_binop(x, y, |a, b| a.pow_assign_round(b, Nearest)),
            sampler,
            SAMPLE_COUNT,
        );
    }
}
