#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::nursery)]
#![warn(missing_docs)]
// `!(x <= bound)` / `!(x >= bound)` are deliberate NaN-propagating domain
// guards — unlike `x > bound`, they route NaN down the reject branch.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use fast_polynomial::poly_array as poly;

/// Real functions for `f32`s
pub mod f32;

/// Real functions for `f64`s
pub mod f64;

/// Explicit sign rather than a `bool`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sign {
    /// Positive
    Positive,

    /// Negative
    Negative,
}

const fn u32_sign_bit(sign: Sign) -> u32 {
    match sign {
        Sign::Positive => 0,
        Sign::Negative => 1 << 31,
    }
}

const fn u64_sign_bit(sign: Sign) -> u64 {
    match sign {
        Sign::Positive => 0,
        Sign::Negative => 1 << 63,
    }
}

/// Fast multiply-add
///
/// This function picks the faster way to compute `x * y + a` depending on the
/// target architecture.  The FMA instruction is used if available.  Otherwise,
/// it falls back to `x * y + a` that is faster but gives less accurate results
/// than a true FMA.
///
/// # Not an error-free transform
///
/// Because the fallback path rounds the product *before* the addition, this
/// helper is **not** fused on every target.  Do not use it where correctness
/// depends on the single rounding of a true FMA — error-free transforms,
/// residual tests, and high-precision compensation must use
/// [`correct_mul_add`].  Reserve this for hot polynomial-style spots where a
/// lost low bit is absorbed by later rounding.
// Not `const`: the hardware path calls the non-const `f64::mul_add`.
#[allow(
    unreachable_code,
    clippy::missing_const_for_fn,
    clippy::disallowed_methods
)]
#[inline]
fn fast_mul_add(x: f64, y: f64, a: f64) -> f64 {
    #[cfg(feature = "_no_fma")]
    #[allow(clippy::suboptimal_flops)]
    return x * y + a;

    // x86/x86_64 without compile-time FMA: runtime dispatch.
    // `is_x86_feature_detected!` caches via an AtomicU8 (one-time CPUID cost),
    // so subsequent calls pay only an atomic load plus a branch the predictor
    // always gets right.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[cfg(not(target_feature = "fma"))]
    {
        #[target_feature(enable = "fma")]
        unsafe fn force_fma(x: f64, y: f64, a: f64) -> f64 {
            x.mul_add(y, a)
        }

        if std::is_x86_feature_detected!("fma") {
            // SAFETY: runtime check confirmed FMA is available on this CPU.
            return unsafe { force_fma(x, y, a) };
        }

        #[allow(clippy::suboptimal_flops)]
        return x * y + a;
    }

    // Every other target (compile-time FMA, aarch64 where fp-armv8 is
    // baseline, wasm32, …): delegate to Rust's `mul_add`, which LLVM
    // lowers correctly.
    x.mul_add(y, a)
}

/// Correctly-rounded multiply-add with runtime FMA dispatch
///
/// Always computes `x * y + a` as a single fused operation.  On `x86`/`x86_64`
/// without a compile-time `+fma` target feature the FMA instruction is
/// selected at runtime; on targets where the FMA instruction is unavailable
/// it falls back to the platform's software `fma` implementation.
///
/// Use this instead of `x.mul_add(y, a)` for error-free transforms, residual
/// tests, and double-double compensation, where the single rounding of a true
/// FMA is required for correctness.  For polynomial hot paths where a lost low
/// bit is acceptable, prefer `fast_mul_add`, which avoids the software `fma`
/// fallback cost on old hardware.
// Not `const`: the hardware path calls the non-const `f64::mul_add`.
#[must_use]
#[allow(
    unreachable_code,
    clippy::missing_const_for_fn,
    clippy::disallowed_methods
)]
#[inline]
pub fn correct_mul_add(x: f64, y: f64, a: f64) -> f64 {
    // x86/x86_64 without compile-time FMA: runtime dispatch to the hardware
    // FMA instruction; fall back to the software `fma` for old CPUs.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[cfg(not(target_feature = "fma"))]
    {
        #[target_feature(enable = "fma")]
        unsafe fn force_fma(x: f64, y: f64, a: f64) -> f64 {
            x.mul_add(y, a)
        }

        if std::is_x86_feature_detected!("fma") {
            // SAFETY: runtime check confirmed FMA is available on this CPU.
            return unsafe { force_fma(x, y, a) };
        }
    }

    x.mul_add(y, a)
}

/// Const evaluation of 2<sup>`n`</sup>
#[inline]
const fn exp2i(n: i64) -> f64 {
    let bits = match n + 1023 {
        2047.. => return f64::INFINITY,
        s @ 1..=2046 => s << f64::EXP_SHIFT,
        s @ -63..=0 => 1 << (f64::EXP_SHIFT - 1) >> -s,
        _ => 0,
    };
    #[allow(clippy::cast_sign_loss)]
    f64::from_bits(bits as u64)
}

#[allow(clippy::float_cmp)]
const _: () = {
    let (mut n, mut x) = (0, 1.0);

    while n < 1100 {
        assert!(exp2i(n) == x);
        x *= 2.0;
        n += 1;
    }

    (n, x) = (0, 1.0);

    while n > -1100 {
        assert!(exp2i(n) == x);
        x *= 0.5;
        n -= 1;
    }
};
