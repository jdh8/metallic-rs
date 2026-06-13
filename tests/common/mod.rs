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
pub fn test_univariate_cases<Case: Copy + LowerExp, Output: Identity + Debug>(
    f: impl Fn(Case) -> Output,
    g: impl Fn(Case) -> Output,
    cases: impl Iterator<Item = Case>,
) {
    truncate_errors(cases.filter_map(|x| {
        let f = f(x);
        let g = g(x);
        (!f.is(&g)).then(|| println!("{x:e}: {f:?} != {g:?}"))
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
