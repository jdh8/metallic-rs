#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::nursery)]
#![warn(missing_docs)]
// `!(x <= bound)` / `!(x >= bound)` are deliberate NaN-propagating domain
// guards — unlike `x > bound`, they route NaN down the reject branch.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use fast_polynomial::poly_array as poly;

mod f32_;
mod f64_;

// Flat libm-style public API: `f64` keeps the bare C name, `f32` gets the `f`
// suffix.  Implementation modules use the same names directly.
pub use f64_::{
    acos, acosh, asin, asinh, atan, atan2, atanh, cbrt, cos, cosh, erf, erfc, exp, exp2, exp10,
    expm1, fma, frexp, hypot, ldexp, lgamma, log, log1p, log2, log10, round, sin, sincos, sinh,
    tan, tanh, tgamma,
};

/// To avoid name collisions with the module [`f64_::pow`].
pub use f64_::pow::pow;

pub use f32_::{
    acosf, acoshf, asinf, asinhf, atan2f, atanf, atanhf, cbrtf, cosf, coshf, erfcf, erff, exp2f,
    exp10f, expf, expm1f, fmaf, frexpf, hypotf, ldexpf, lgammaf, log1pf, log2f, log10f, logf, powf,
    roundf, sincosf, sinf, sinhf, tanf, tanhf, tgammaf,
};

// Crate-internal `f64` primitives, reached as `crate::exp2i` / `crate::fast_mul_add`
// from both precision trees (private re-export, like `poly` above).
use f64_::{exp2i, fast_mul_add};

/// Explicit sign rather than a `bool`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sign {
    /// Positive
    Positive,

    /// Negative
    Negative,
}
