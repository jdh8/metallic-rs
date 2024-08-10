/// Explicitly stored significand bits in [`prim@f64`]
///
/// This constant is usually used as a shift to access the exponent bits.
pub const EXP_SHIFT: u32 = f64::MANTISSA_DIGITS - 1;

/// Fast multiply-add
///
/// This function picks the faster way to compute `x * y + a` depending on the
/// target architecture.  The FMA instruction is used if available.  Otherwise,
/// it falls back to `x * y + a` that is faster but gives less accurate results
/// than [`f64::mul_add`]
#[must_use]
#[inline]
pub fn mul_add(x: f64, y: f64, a: f64) -> f64 {
    #[cfg(target_feature = "fma")]
    return x.mul_add(y, a);

    #[cfg(not(target_feature = "fma"))]
    return x * y + a;
}

/// Polynomial evaluation with Horner's method
///
/// This function evaluates a polynomial with coefficients in `p` at `x`.
/// This function calls [`mul_add`] for simplicity.
#[must_use]
#[inline]
pub fn poly(x: f64, p: &[f64]) -> f64 {
    p.iter()
        .copied()
        .rev()
        .reduce(|y, c| mul_add(y, x, c))
        .unwrap_or_default()
}
