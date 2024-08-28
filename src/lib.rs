#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
use fast_polynomial::poly_array as poly;

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

/// Const evaluation of 2<sup>`n`</sup>
#[inline]
const fn exp2i(n: i64) -> f64 {
    let bits = match n + 1023 {
        2047.. => return f64::INFINITY,
        s @ 1..=2046 => s << f64::EXP_SHIFT,
        s @ -63..=0 => 1 << (f64::EXP_SHIFT - 1) >> -s,
        _ => 0,
    };
    unsafe { core::mem::transmute(bits) }
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
