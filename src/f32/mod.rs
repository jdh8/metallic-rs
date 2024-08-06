mod kernel;

/// The least number greater than `x`
///
/// This is a less careful version of [`f32::next_up`] regarding subnormal
/// numbers.  This function is useful until `f32::next_up` gets stabilized.
#[must_use]
pub fn next_up(x: f32) -> f32 {
    if x.is_nan() || x == f32::INFINITY {
        x
    } else if x == 0.0 {
        f32::from_bits(1)
    } else if x.is_sign_negative() {
        f32::from_bits(x.to_bits() - 1)
    } else {
        f32::from_bits(x.to_bits() + 1)
    }
}

/// The greatest number less than `x`
///
/// This is a less careful version of [`f32::next_down`] regarding subnormal
/// numbers.  This function is useful until `f32::next_down` gets stabilized.
#[must_use]
pub fn next_down(x: f32) -> f32 {
    if x.is_nan() || x == f32::NEG_INFINITY {
        x
    } else if x == 0.0 {
        f32::from_bits(0x8000_0001)
    } else if x.is_sign_negative() {
        f32::from_bits(x.to_bits() + 1)
    } else {
        f32::from_bits(x.to_bits() - 1)
    }
}

/// The exponential function
#[must_use]
pub fn exp(x: f32) -> f32 {
    use core::f32::consts::LN_2;
    use core::f64::consts;

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
    if x < (f32::MIN_EXP - f32::MANTISSA_DIGITS as i32 - 1) as f32 * LN_2 {
        return 0.0;
    }

    #[allow(clippy::cast_precision_loss)]
    if x > f32::MAX_EXP as f32 * LN_2 {
        return f32::INFINITY;
    }

    let x = f64::from(x);
    let n = (x * consts::LOG2_E).round_ties_even();
    let x = n.mul_add(-consts::LN_2, x);
    let y = kernel::exp(x).mul_add(x, 1.0);

    #[allow(clippy::cast_possible_truncation)]
    return kernel::fast_ldexp(y, n as i64) as f32;
}

/// Multiply `x` by 2 raised to the power of `n`
#[must_use]
pub fn ldexp(x: f32, n: i32) -> f32 {
    let coefficient = match n {
        ..-1022 => f64::exp2(-1023.0),

        #[allow(clippy::cast_sign_loss)]
        n @ -1022..1024 => f64::from_bits(((n + 1023) as u64) << 52),

        1024.. => f64::MAX,
    };

    #[allow(clippy::cast_possible_truncation)]
    return (f64::from(x) * coefficient) as f32;
}
