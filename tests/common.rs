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
