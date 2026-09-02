//! One integration-test binary: every file under `tests/` linked once instead
//! of once per function.  Test names are `f64_::sin::test_sin`, so filter a
//! function with `cargo test --test all -- '::sin::'` (the `::` anchors keep
//! `sin` from also selecting `asin`).
#![cfg_attr(feature = "f128", feature(f128))]

mod common;
#[cfg(feature = "f128")]
#[path = "common/f128.rs"]
mod common128;
#[cfg(feature = "f128")]
#[path = "common/f128_exp.rs"]
mod common_exp;

mod f32_ {
    mod acosf;
    mod acoshf;
    mod acospif;
    mod asinf;
    mod asinhf;
    mod asinpif;
    mod atan2f;
    mod atan2pif;
    mod atanf;
    mod atanhf;
    mod atanpif;
    mod cbrtf;
    mod compoundf;
    mod cosf;
    mod coshf;
    mod cospif;
    mod erfcf;
    mod erff;
    mod exp10f;
    mod exp10m1f;
    mod exp2f;
    mod exp2m1f;
    mod expf;
    mod expm1f;
    mod frexpf;
    mod hypotf;
    mod ldexpf;
    mod lgammaf;
    mod log10f;
    mod log10p1f;
    mod log1pf;
    mod log2f;
    mod log2p1f;
    mod logf;
    mod powf;
    mod roundf;
    mod rsqrtf;
    mod sincosf;
    mod sinf;
    mod sinhf;
    mod sinpif;
    mod tanf;
    mod tanhf;
    mod tanpif;
    mod tgammaf;
}

mod f64_ {
    mod acos;
    mod acosh;
    mod acospi;
    mod asin;
    mod asinh;
    mod asinpi;
    mod atan;
    mod atan2;
    mod atan2pi;
    mod atanh;
    mod atanpi;
    mod cbrt;
    mod compound;
    mod cos;
    mod cosh;
    mod cospi;
    mod erf;
    mod erfc;
    mod exp;
    mod exp10;
    mod exp10m1;
    mod exp2;
    mod exp2m1;
    mod expm1;
    mod frexp;
    mod gen_sinpi_table;
    mod hypot;
    mod ldexp;
    mod lgamma;
    mod log;
    mod log10;
    mod log10p1;
    mod log1p;
    mod log2;
    mod log2p1;
    mod pow;
    mod rsqrt;
    mod sin;
    mod sincos;
    mod sinh;
    mod sinpi;
    mod tan;
    mod tanh;
    mod tanpi;
    mod tgamma;
}

#[cfg(feature = "f128")]
mod f128_ {
    mod acosq;
    mod asinq;
    mod atan2q;
    mod atanq;
    mod cbrtq;
    mod cosq;
    mod exp10q;
    mod exp2q;
    mod expm1q;
    mod expq;
    mod hypotq;
    mod log2q;
    mod logq;
    mod rsqrtq;
    mod sinq;
    mod sqrtq;
    mod tanq;
}
