// Rust analyzer reports false positive for every function.
// -- jdh8, 2025-05-11
#![allow(dead_code)]

use core::fmt::{Debug, LowerExp};

/// Semantic identity like `Object.is` in JavaScript
///
/// This function works around comparison issues with NaNs and signed zeros.
/// To be specific, `is(f64::NAN, f64::NAN)` but not `is(0.0, -0.0)`.
pub trait Identity {
    fn is(&self, other: &Self) -> bool;
}

impl Identity for f32 {
    fn is(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits() || (self.is_nan() && other.is_nan())
    }
}

impl Identity for f64 {
    fn is(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits() || (self.is_nan() && other.is_nan())
    }
}

impl<T: Identity, U: Identity> Identity for (T, U) {
    fn is(&self, other: &Self) -> bool {
        self.0.is(&other.0) && self.1.is(&other.1)
    }
}

fn truncate_errors(errors: impl Iterator<Item = ()>) {
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

/// Exhaustively test for every `u32` value
///
/// - `error`: function returning `Some` if there is an error
pub fn exhaustively_test_u32(error: impl Fn(u32) -> Option<()>) {
    const LIMIT: usize = 250;
    let count = (0..=u32::MAX).filter_map(error).take(LIMIT).count();

    assert!(
        count < LIMIT,
        "Too many (>= {LIMIT}) mismatches!  Aborting...",
    );
    assert!(count == 0, "There are {count} mismatches");
}

/// Check if `f` returns the same result as `g` for every `f32` values
///
/// By "same result", I mean semantic identity as defined by [`is`].
pub fn test_identity<T: Identity + core::fmt::Debug>(f: impl Fn(f32) -> T, g: impl Fn(f32) -> T) {
    exhaustively_test_u32(|i| {
        let x = f32::from_bits(i);
        let f = f(x);
        let g = g(x);

        (!f.is(&g)).then(|| println!("{x:e}: {f:?} != {g:?}"))
    });
}

/// Check if `result` is within the nearby `f32` representations of `expected`
///
/// Due to [the Table Maker's Dilemma][dilemma], it is infeasible to implement a
/// correctly-rounded (error < 0.5 ulp) transcendental function.  However,
/// faithful rounding (error < 1 ulp) is usually achievable.
///
/// [dilemma]: https://hal-lara.archives-ouvertes.fr/hal-02101765/document
///
/// If `expected` has an exact `f32` representation, `result` must be that
/// value.  Otherwise, `expected` has two `f32` neighbors, and `result` must be
/// either of them.
pub fn is_faithful_rounding(result: f32, expected: f64) -> bool {
    #[allow(clippy::cast_possible_truncation)]
    if result.is(&(expected as f32)) {
        return true;
    }

    let next_up = f64::from(metallic::f32::next_up(result));
    let next_down = f64::from(metallic::f32::next_down(result));
    next_down < expected && expected < next_up
}

// Code repetition is intentional for future removal of this function
pub fn test_bivariate_faithful(f: impl Fn(f32, f32) -> f32, g: impl Fn(f64, f64) -> f64) {
    exhaustively_test_u32(|bits| {
        let x = f32::from_bits(0x10001 * (bits >> 16));
        let y = f32::from_bits(bits << 16);
        let f = f(x, y);
        let g = g(x.into(), y.into());

        (!is_faithful_rounding(f, g)).then(|| println!("{x:e}, {y:e}: {f:e} != {g:e}"))
    });
}

pub fn test_bivariate_correct(f: impl Fn(f32, f32) -> f32, g: impl Fn(f32, f32) -> f32) {
    exhaustively_test_u32(|bits| {
        let x = f32::from_bits(0x10001 * (bits >> 16));
        let y = f32::from_bits(bits << 16);
        let f = f(x, y);
        let g = g(x, y);

        (!f.is(&g)).then(|| println!("{x:e}, {y:e}: {f:e} != {g:e}"))
    });
}
