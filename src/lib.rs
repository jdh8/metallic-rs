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
// suffix.  The implementation modules keep Rust-flavoured names internally; the
// C names live only on these crate-root re-exports.
pub use f64_::{
    acos, acosh, asin, asinh, atan, atan2, atanh, cbrt, cos, cosh, erf, erfc, exp, exp_m1 as expm1,
    exp2, exp10, frexp, hypot, ldexp, lgamma, ln as log, ln_1p as log1p, log2, log10, powf as pow,
    round, sin, sin_cos as sincos, sinh, tan, tanh, tgamma,
};

pub use f32_::{
    acos as acosf, acosh as acoshf, asin as asinf, asinh as asinhf, atan as atanf, atan2 as atan2f,
    atanh as atanhf, cbrt as cbrtf, cos as cosf, cosh as coshf, erf as erff, erfc as erfcf,
    exp as expf, exp_m1 as expm1f, exp2 as exp2f, exp10 as exp10f, frexp as frexpf,
    hypot as hypotf, ldexp as ldexpf, lgamma as lgammaf, ln as logf, ln_1p as log1pf,
    log2 as log2f, log10 as log10f, powf, round as roundf, sin as sinf, sin_cos as sincosf,
    sinh as sinhf, tan as tanf, tanh as tanhf, tgamma as tgammaf,
};

// Correctly-rounded fused multiply-add, exposed under their C names.
pub use f32_::fmaf;
pub use f64_::fma;

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
