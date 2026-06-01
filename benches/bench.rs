#![allow(dead_code)]

/// The sign bit of an `f32`.
const SIGN: u32 = 0x8000_0000;

/// Flip an `f32` (sign-magnitude) into a `u32` that increases monotonically
/// with the float's value, so that a contiguous integer interval corresponds to
/// a contiguous run of representations.  Inverse: [`from_ordered`].
fn to_ordered(x: f32) -> u32 {
    let bits = x.to_bits();
    bits ^ (((bits as i32) >> 31) as u32 | SIGN)
}

/// Inverse of [`to_ordered`].
fn from_ordered(key: u32) -> f32 {
    f32::from_bits(key ^ ((key >> 31).wrapping_sub(1) | SIGN))
}

/// A uniformly random `f32`: every one of the 2^32 bit patterns is equally
/// likely (subnormals, ±∞, and NaN included), matching `test_all_f32`.
///
/// Unlike `rand::random::<f32>()`, which is uniform on `[0, 1)`.
pub fn random_f32() -> f32 {
    f32::from_bits(rand::random())
}

/// A uniformly random `f32` among the representations in `range`: every bit
/// pattern between the endpoints is equally likely (sign-magnitude order).
pub fn random_f32_in(range: core::ops::RangeInclusive<f32>) -> f32 {
    let lo = to_ordered(*range.start());
    let hi = to_ordered(*range.end());
    from_ordered(rand::random_range(lo..=hi))
}

/// A uniformly random `f64`: every one of the 2^64 bit patterns is equally
/// likely (subnormals, ±∞, and NaN included).
pub fn random_f64() -> f64 {
    f64::from_bits(rand::random())
}

/// Define a Criterion bench function `$name` timing `$callback` on random input.
///
/// Each iteration draws fresh input.  Placeholders:
/// - `_` — a uniformly random `f32` over every bit pattern ([`random_f32`])
/// - `in $range` — a uniformly random `f32` among the representations in
///   `$range` ([`random_f32_in`]); the range is `RangeInclusive`
/// - any expression — passed through verbatim (drawn per iteration)
#[macro_export]
macro_rules! bench {
    ($name:ident, $callback:expr $(, $args:expr)* $(,)?) => {
        fn $name(criterion: &mut criterion::Criterion) {
            criterion.bench_function(stringify!($callback), |bencher| {
                bencher.iter(|| $callback($($args),*));
            });
        }
    };
    ($name:ident, $callback:expr, _, _ $(,)?) => {
        bench!($name, $callback, bench::random_f32(), bench::random_f32());
    };
    ($name:ident, $callback:expr, in $a:expr, in $b:expr $(,)?) => {
        bench!($name, $callback, bench::random_f32_in($a), bench::random_f32_in($b));
    };
    ($name:ident, $callback:expr, _ $(,)?) => {
        bench!($name, $callback, bench::random_f32());
    };
    ($name:ident, $callback:expr, in $range:expr $(,)?) => {
        bench!($name, $callback, bench::random_f32_in($range));
    };
}
