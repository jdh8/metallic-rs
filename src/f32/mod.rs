mod kernel;

/// The exponential function
#[must_use]
pub fn exp(x: f32) -> f32 {
    use core::f64::consts;
    let x = f64::from(x);

    if x <= -150.0 * consts::LN_2 {
        return 0.0;
    }

    if x >= 128.0 * consts::LN_2 {
        return f32::INFINITY;
    }

    let n = (x * consts::LOG2_E).round_ties_even();
    let y = kernel::exp(n.mul_add(-consts::LN_2, x)).mul_add(x, 1.0);

    #[allow(clippy::cast_possible_truncation)]
    return kernel::fast_ldexp(y, n as i32) as f32;
}