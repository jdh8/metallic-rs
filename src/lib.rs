#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::nursery)]
#![warn(missing_docs)]
// FMA discipline: never call the builtin `mul_add` directly and never hand-write
// a raw `a * b + c`.  Use `crate::fma`/`crate::fmaf` for exact (error-free)
// transforms, or `crate::fast_mul_add` for hot polynomial spots.
#![deny(clippy::suboptimal_flops, clippy::disallowed_methods)]
// `!(x <= bound)` / `!(x >= bound)` are deliberate NaN-propagating domain
// guards — unlike `x > bound`, they route NaN down the reject branch.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use fast_polynomial::poly_array as poly;

mod f32_;
mod f64_;

// Flat libm-style public API: `f64` keeps the bare C name, `f32` gets the `f`
// suffix.  Implementation modules use the same names directly.
pub use f64_::{
    acos, acosh, acospi, asin, asinh, asinpi, atan, atan2, atanh, cbrt, cos, cosh, cospi, erf,
    erfc, exp, exp2, exp10, expm1, fma, frexp, hypot, ldexp, lgamma, log, log1p, log2, log10,
    round, rsqrt, sin, sincos, sinh, sinpi, tan, tanh, tanpi, tgamma,
};

/// To avoid name collisions with the module [`f64_::pow`].
pub use f64_::pow::pow;

pub use f32_::{
    acosf, acoshf, acospif, asinf, asinhf, asinpif, atan2f, atanf, atanhf, cbrtf, cosf, coshf,
    cospif, erfcf, erff, exp2f, exp10f, expf, expm1f, fmaf, frexpf, hypotf, ldexpf, lgammaf,
    log1pf, log2f, log10f, logf, powf, roundf, rsqrtf, sincosf, sinf, sinhf, sinpif, tanf, tanhf,
    tanpif, tgammaf,
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
