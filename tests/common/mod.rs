// Rust analyzer reports false positive for every function.
// -- jdh8, 2025-05-11
#![allow(dead_code)]

use core::fmt::{Debug, LowerExp};
use std::io::BufRead as _;
use std::path::{Path, PathBuf};

/// Semantic identity like `Object.is` in JavaScript
///
/// This function works around comparison issues with NaNs and signed zeros.
/// To be specific, `is(f64::NAN, f64::NAN)` but not `is(0.0, -0.0)`.
pub trait Identity {
    fn is(&self, other: &Self) -> bool;
}

impl Identity for f32 {
    #[inline]
    fn is(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits() || (self.is_nan() && other.is_nan())
    }
}

impl Identity for f64 {
    #[inline]
    fn is(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits() || (self.is_nan() && other.is_nan())
    }
}

impl<T: Identity, U: Identity> Identity for (T, U) {
    fn is(&self, other: &Self) -> bool {
        self.0.is(&other.0) && self.1.is(&other.1)
    }
}

/// Truncate error reporting iterator to reasonable length
///
/// This library aims for correct rounding.  Reporting thousands of cases does
/// not help much.  Currently, this function limits the report to 250 cases.
pub fn truncate_errors(errors: impl Iterator) {
    const LIMIT: usize = 250;
    let count = errors.take(LIMIT).count();

    assert!(
        count < LIMIT,
        "Too many (>= {LIMIT}) mismatches!  Aborting...",
    );
    assert!(count == 0, "There are {count} mismatches");
}

/// Signed-magnitude ordering of an `f32`, so that adjacent floats differ by one
fn ordered_f32(x: f32) -> i64 {
    let magnitude = i64::from(x.to_bits() & 0x7fff_ffff);
    if x.is_sign_negative() {
        -magnitude
    } else {
        magnitude
    }
}

/// ulp error between two `f32`s, with NaN and ∞ required to match exactly
pub fn ulp_error_f32(a: f32, b: f32) -> u64 {
    if a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan()) {
        return 0;
    }
    if a.is_nan() || b.is_nan() || a.is_infinite() || b.is_infinite() {
        return u64::MAX;
    }
    (ordered_f32(a) - ordered_f32(b)).unsigned_abs()
}

/// Signed-magnitude ordering of an `f64`, so that adjacent floats differ by one
fn ordered_f64(x: f64) -> i128 {
    let magnitude = i128::from(x.to_bits() & 0x7fff_ffff_ffff_ffff);
    if x.is_sign_negative() {
        -magnitude
    } else {
        magnitude
    }
}

/// ulp error between two `f64`s.  NaN must match NaN exactly (else `u64::MAX`);
/// ∞ is treated as the ordinary code point just past `MAX`, so the overflow
/// boundary `MAX`↔∞ counts as 1 ulp — the right metric for a faithful floor.
pub fn ulp_error_f64(a: f64, b: f64) -> u64 {
    if a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan()) {
        return 0;
    }
    if a.is_nan() || b.is_nan() {
        return u64::MAX;
    }
    u64::try_from((ordered_f64(a) - ordered_f64(b)).unsigned_abs()).unwrap_or(u64::MAX)
}

/// Like [`test_univariate_cases`] but tolerant up to `tol` ulps
///
/// This is for faithfully-rounded (≤ 1 ulp) functions that are not yet
/// correctly rounded.
pub fn test_univariate_faithful<Case: Copy + LowerExp>(
    f: impl Fn(Case) -> f32,
    g: impl Fn(Case) -> f32,
    cases: impl Iterator<Item = Case>,
    tol: u64,
) {
    truncate_errors(cases.filter_map(|x| {
        let (f, g) = (f(x), g(x));
        let error = ulp_error_f32(f, g);
        (error > tol).then(|| println!("{x:e}: {f:e} != {g:e} ({error} ulp)"))
    }));
}

/// Check if `f` returns the same result as `g` for the provided cases
///
/// By "same result", I mean semantic identity as defined by [`is`].
pub fn test_univariate_cases<Case: Copy + Debug, Output: Identity + Debug>(
    f: impl Fn(Case) -> Output,
    g: impl Fn(Case) -> Output,
    cases: impl Iterator<Item = Case>,
) {
    truncate_errors(cases.filter_map(|x| {
        let f = f(x);
        let g = g(x);
        (!f.is(&g)).then(|| println!("{x:?}: {f:?} != {g:?}"))
    }));
}

/// Check if `f` returns the same result as `g` for every `f32` value
///
/// By "same result", I mean semantic identity as defined by [`is`].
pub fn test_all_f32<Output: Identity + Debug>(
    f: impl Fn(f32) -> Output,
    g: impl Fn(f32) -> Output,
) {
    test_univariate_cases(f, g, (0..=u32::MAX).map(f32::from_bits));
}

/// Check if `f` returns the same result as `g` for the provided pairs
///
/// By "same result", I mean semantic identity as defined by [`is`].  Generic
/// over the input and output type, so it serves both `f32` and `f64` bivariate
/// functions (`hypot`, `atan2`, `powf`).
pub fn test_bivariate_cases<In: Copy + LowerExp, Out: Identity + Debug>(
    f: impl Fn(In, In) -> Out,
    g: impl Fn(In, In) -> Out,
    cases: impl Iterator<Item = [In; 2]>,
) {
    truncate_errors(cases.filter_map(|[x, y]| {
        let f = f(x, y);
        let g = g(x, y);
        (!f.is(&g)).then(|| println!("{x:e}, {y:e}: {f:?} != {g:?}"))
    }));
}

/// CORE-MATH worst-case gate: check `f` bit-exact against `oracle` on the
/// hard-to-round corpus `tests/cases/<name>.wc`.  This reproduces CORE-MATH's
/// `--worst` step in round-to-nearest.  Passes vacuously when the corpus file is
/// absent, so a checkout without the corpora still builds.
pub fn test_worst_univariate<Out: Identity + Debug>(
    name: &str,
    f: impl Fn(f64) -> Out,
    oracle: impl Fn(f64) -> Out,
) {
    test_univariate_cases(f, oracle, parse_case_file(format!("{name}.wc"), parse_f64));
}

/// Bivariate [`test_worst_univariate`], for `atan2`, `hypot`, `pow`.
pub fn test_worst_bivariate<Out: Identity + Debug>(
    name: &str,
    f: impl Fn(f64, f64) -> Out,
    oracle: impl Fn(f64, f64) -> Out,
) {
    test_bivariate_cases(
        f,
        oracle,
        parse_case_file(format!("{name}.wc"), parse_f64_pair),
    );
}

/// Like [`test_worst_univariate`] but tolerant up to `tol` ulps: the blocking
/// floor for functions that are faithfully rounded (≤ 1 ulp) but not yet
/// correctly rounded (their strict gate is `#[ignore]`d pending the CR push).
pub fn test_worst_faithful(
    name: &str,
    f: impl Fn(f64) -> f64,
    oracle: impl Fn(f64) -> f64,
    tol: u64,
) {
    truncate_errors(
        parse_case_file(format!("{name}.wc"), parse_f64).filter_map(|x| {
            let (a, b) = (f(x), oracle(x));
            let error = ulp_error_f64(a, b);
            (error > tol).then(|| println!("{x:e}: {a:e} != {b:e} ({error} ulp)"))
        }),
    );
}

/// Bivariate [`test_worst_faithful`].
pub fn test_worst_faithful_bivariate(
    name: &str,
    f: impl Fn(f64, f64) -> f64,
    oracle: impl Fn(f64, f64) -> f64,
    tol: u64,
) {
    truncate_errors(
        parse_case_file(format!("{name}.wc"), parse_f64_pair).filter_map(|[x, y]| {
            let (a, b) = (f(x, y), oracle(x, y));
            let error = ulp_error_f64(a, b);
            (error > tol).then(|| println!("{x:e}, {y:e}: {a:e} != {b:e} ({error} ulp)"))
        }),
    );
}

/// SplitMix64 hash — deterministic pseudo-random `u64` for sampling.
pub fn mix64(i: u64) -> u64 {
    let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Value-uniform `f64` in `[lo, hi]` drawn from a 64-bit hash.
pub fn uniform(hash: u64, lo: f64, hi: f64) -> f64 {
    let unit = (hash >> 11) as f64 / (1u64 << 53) as f64; // [0, 1)
    metallic::fma(unit, hi - lo, lo)
}

/// Independent gold-standard cross-check (CORE-MATH uses MPFR as its reference):
/// bit-exact `f` vs a correctly-rounded MPFR result `cr` over `n` samples drawn
/// by `sampler`.  Strict round-to-nearest, so it is RED for functions that are
/// only faithfully rounded — run deliberately with `--features mpfr`.
#[cfg(feature = "mpfr")]
pub fn mpfr_sweep_univariate(
    f: impl Fn(f64) -> f64,
    cr: impl Fn(f64) -> f64,
    sampler: impl Fn(u64) -> f64,
    n: u64,
) {
    truncate_errors((0..n).filter_map(|i| {
        let x = sampler(i);
        let (got, want) = (f(x), cr(x));
        (!got.is(&want)).then(|| println!("{x:e}: {got:e} != {want:e} (correct)"))
    }));
}

/// Bivariate [`mpfr_sweep_univariate`].
#[cfg(feature = "mpfr")]
pub fn mpfr_sweep_bivariate(
    f: impl Fn(f64, f64) -> f64,
    cr: impl Fn(f64, f64) -> f64,
    sampler: impl Fn(u64) -> [f64; 2],
    n: u64,
) {
    truncate_errors((0..n).filter_map(|i| {
        let [x, y] = sampler(i);
        let (got, want) = (f(x, y), cr(x, y));
        (!got.is(&want)).then(|| println!("{x:e}, {y:e}: {got:e} != {want:e} (correct)"))
    }));
}

pub fn parse_case_file<T, E>(
    filename: impl AsRef<Path>,
    mut parse: impl FnMut(&str) -> Result<T, E>,
) -> impl Iterator<Item = T> {
    std::fs::File::open(PathBuf::from("tests/cases/").join(filename))
        .map(std::io::BufReader::new)
        .map(|stream| {
            stream
                .lines()
                .map_while(Result::ok)
                .filter_map(move |line| {
                    let line = line[..line.find('#').unwrap_or(line.len())].trim_ascii();
                    parse(line).ok()
                })
        })
        .into_iter()
        .flatten()
}

pub enum ParsePairError {
    EmptyField,
    Hexf,
}

impl From<hexf_parse::ParseHexfError> for ParsePairError {
    fn from(_: hexf_parse::ParseHexfError) -> Self {
        Self::Hexf
    }
}

pub fn parse_f32(s: &str) -> Result<f32, hexf_parse::ParseHexfError> {
    fn fallback(s: &str) -> Option<f32> {
        match s {
            "snan" => Some(f32::from_bits(f32::NAN.to_bits() | 1)),
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            s if s.starts_with("0x") => u32::from_str_radix(&s[2..], 16)
                .ok()
                .map(|x| x as f32)
                // Hex floats beyond f32's range or precision (CORE-MATH `.wc`
                // files contain e.g. `0x1p-1022`): parse exactly as f64, then
                // round once to f32, matching C's `strtof`.
                .or_else(|| hexf_parse::parse_hexf64(s, true).ok().map(|x| x as f32)),
            _ => None,
        }
    }

    match hexf_parse::parse_hexf32(s, true) {
        Ok(value) => Ok(value),
        Err(e) => s.parse().or_else(|_| {
            match s.bytes().next() {
                Some(b'+') => fallback(&s[1..]),
                Some(b'-') => fallback(&s[1..]).map(core::ops::Neg::neg),
                _ => fallback(s),
            }
            .ok_or(e)
        }),
    }
}

pub fn parse_f32_pair(s: &str) -> Result<[f32; 2], ParsePairError> {
    let mut fields = s.splitn(2, ',').map(str::trim_ascii);
    let x = parse_f32(fields.next().ok_or(ParsePairError::EmptyField)?)?;
    let y = parse_f32(fields.next().ok_or(ParsePairError::EmptyField)?)?;
    Ok([x, y])
}

pub fn parse_f32_triple(s: &str) -> Result<[f32; 3], ParsePairError> {
    let mut fields = s.splitn(3, ',').map(str::trim_ascii);
    let x = parse_f32(fields.next().ok_or(ParsePairError::EmptyField)?)?;
    let y = parse_f32(fields.next().ok_or(ParsePairError::EmptyField)?)?;
    let z = parse_f32(fields.next().ok_or(ParsePairError::EmptyField)?)?;
    Ok([x, y, z])
}

pub fn parse_f64_pair(s: &str) -> Result<[f64; 2], ParsePairError> {
    let mut fields = s.splitn(2, ',').map(str::trim_ascii);
    let x = parse_f64(fields.next().ok_or(ParsePairError::EmptyField)?)?;
    let y = parse_f64(fields.next().ok_or(ParsePairError::EmptyField)?)?;
    Ok([x, y])
}

pub fn parse_f64_triple(s: &str) -> Result<[f64; 3], ParsePairError> {
    let mut fields = s.splitn(3, ',').map(str::trim_ascii);
    let x = parse_f64(fields.next().ok_or(ParsePairError::EmptyField)?)?;
    let y = parse_f64(fields.next().ok_or(ParsePairError::EmptyField)?)?;
    let z = parse_f64(fields.next().ok_or(ParsePairError::EmptyField)?)?;
    Ok([x, y, z])
}

pub fn parse_f64(s: &str) -> Result<f64, hexf_parse::ParseHexfError> {
    fn fallback(s: &str) -> Option<f64> {
        match s {
            "snan" => Some(f64::from_bits(f64::NAN.to_bits() | 1)),
            // hexf parses only exactly-representable forms; corpus rows may
            // carry an empty integer part (`0x.fffp-1022`) or a subnormal
            // whose 53-bit mantissa needs rounding (`0x1.45f306dc9c882p-1024`).
            // Rebuild a ≤53-bit mantissa exactly and let `ldexp` round once.
            s if s.starts_with("0x") && s.contains('p') => {
                let (mant, exp) = s[2..].split_once('p')?;
                let frac_len = mant.split_once('.').map_or(0, |(_, frac)| frac.len());
                let digits = mant.replace('.', "");
                let mantissa = u64::from_str_radix(&digits, 16).ok()?;
                (mantissa < 1 << 53).then_some(())?;
                let exp = exp.parse::<i32>().ok()?;
                #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
                Some(metallic::ldexp(mantissa as f64, exp - 4 * frac_len as i32))
            }
            #[allow(clippy::cast_precision_loss)]
            s if s.starts_with("0x") => u64::from_str_radix(&s[2..], 16).ok().map(|x| x as f64),
            _ => None,
        }
    }

    match hexf_parse::parse_hexf64(s, true) {
        Ok(value) => Ok(value),
        Err(e) => s.parse().or_else(|_| {
            match s.bytes().next() {
                Some(b'+') => fallback(&s[1..]),
                Some(b'-') => fallback(&s[1..]).map(core::ops::Neg::neg),
                _ => fallback(s),
            }
            .ok_or(e)
        }),
    }
}
