#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Real functions for `f32`s
pub mod f32;

/// Real functions for `f64`s
pub mod f64;

/// Fast multiply-add
///
/// This function picks the faster way to compute `x * y + a` depending on the
/// target architecture.  The FMA instruction is used if available.  Otherwise,
/// it falls back to `x * y + a` that is faster but gives less accurate results
/// than [`f64::mul_add`]
#[inline]
fn mul_add(x: f64, y: f64, a: f64) -> f64 {
    #[cfg(target_feature = "fma")]
    return x.mul_add(y, a);

    #[cfg(not(target_feature = "fma"))]
    #[allow(clippy::suboptimal_flops)]
    return x * y + a;
}

/// Polynomial evaluation with Horner's method
///
/// This function evaluates a polynomial with coefficients in `p` at `x`.
/// This function calls [`mul_add`] for simplicity.
fn poly<const N: usize>(x: f64, p: &[f64; N]) -> f64 {
    #[cfg(target_feature = "fma")]
    return p
        .iter()
        .copied()
        .rev()
        .reduce(|y, c| mul_add(y, x, c))
        .unwrap_or_default();

    #[cfg(not(target_feature = "fma"))]
    return fast_polynomial::poly_array(x, p);
}
