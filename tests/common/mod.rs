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
