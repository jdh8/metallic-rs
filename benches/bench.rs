#![allow(dead_code)]

/// The sign bit of an `f32`.
const SIGN: u32 = 0x8000_0000;

/// The sign bit of an `f64`.
const SIGN_64: u64 = 0x8000_0000_0000_0000;

/// Flip an `f32` (sign-magnitude) into a `u32` that increases monotonically
/// with the float's value, so that a contiguous integer interval corresponds to
/// a contiguous run of representations.  Inverse: [`from_ordered`].
const fn to_ordered(x: f32) -> u32 {
    let bits = x.to_bits();
    bits ^ (((bits as i32) >> 31) as u32 | SIGN)
}

/// Inverse of [`to_ordered`].
const fn from_ordered(key: u32) -> f32 {
    f32::from_bits(key ^ ((key >> 31).wrapping_sub(1) | SIGN))
}

/// [`to_ordered`] for `f64`.
const fn to_ordered_64(x: f64) -> u64 {
    let bits = x.to_bits();
    bits ^ (((bits as i64) >> 63) as u64 | SIGN_64)
}

/// Inverse of [`to_ordered_64`].
const fn from_ordered_64(key: u64) -> f64 {
    f64::from_bits(key ^ ((key >> 63).wrapping_sub(1) | SIGN_64))
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
pub fn random_f32_repr(range: core::ops::RangeInclusive<f32>) -> f32 {
    let lo = to_ordered(*range.start());
    let hi = to_ordered(*range.end());
    from_ordered(rand::random_range(lo..=hi))
}

/// A uniformly random `f64`: every one of the 2^64 bit patterns is equally
/// likely (subnormals, ±∞, and NaN included).
pub fn random_f64() -> f64 {
    f64::from_bits(rand::random())
}

/// A uniformly random `f64` among the representations in `range`: every bit
/// pattern between the endpoints is equally likely (sign-magnitude order).
pub fn random_f64_repr(range: core::ops::RangeInclusive<f64>) -> f64 {
    let lo = to_ordered_64(*range.start());
    let hi = to_ordered_64(*range.end());
    from_ordered_64(rand::random_range(lo..=hi))
}

/// Draw one benchmark input described by a range.  The *form* of the range
/// picks the distribution:
///
/// - a **bounded** range (`a..=b`, `a..b`) is **value-uniform** — every real
///   value in the interval is equally likely.  Right for bounded domains such
///   as `-1.0..=1.0`, where representation-uniform sampling would over-weight
///   tiny magnitudes.
/// - an **open-ended** range (`a..`, `..b`, `..`) is **representation-uniform**
///   — every bit pattern is equally likely ([`random_f32_repr`] / [`random_f32`]
///   / [`random_f64`]).  Right for unbounded domains, where it gives each
///   magnitude an equal chance.
pub trait Draw<T> {
    fn draw(self) -> T;
}

// Bounded → value-uniform.
impl Draw<f32> for core::ops::RangeInclusive<f32> {
    fn draw(self) -> f32 {
        rand::random_range(self)
    }
}
impl Draw<f32> for core::ops::Range<f32> {
    fn draw(self) -> f32 {
        rand::random_range(self)
    }
}

// Open-ended → representation-uniform.
impl Draw<f32> for core::ops::RangeFrom<f32> {
    fn draw(self) -> f32 {
        random_f32_repr(self.start..=f32::INFINITY)
    }
}
impl Draw<f32> for core::ops::RangeTo<f32> {
    fn draw(self) -> f32 {
        random_f32_repr(f32::NEG_INFINITY..=self.end)
    }
}
impl Draw<f32> for core::ops::RangeFull {
    fn draw(self) -> f32 {
        random_f32()
    }
}
impl Draw<f64> for core::ops::RangeFull {
    fn draw(self) -> f64 {
        random_f64()
    }
}

// Bounded → value-uniform.
impl Draw<f64> for core::ops::RangeInclusive<f64> {
    fn draw(self) -> f64 {
        rand::random_range(self)
    }
}
impl Draw<f64> for core::ops::Range<f64> {
    fn draw(self) -> f64 {
        rand::random_range(self)
    }
}

// Open-ended → representation-uniform.
impl Draw<f64> for core::ops::RangeFrom<f64> {
    fn draw(self) -> f64 {
        random_f64_repr(self.start..=f64::INFINITY)
    }
}
impl Draw<f64> for core::ops::RangeTo<f64> {
    fn draw(self) -> f64 {
        random_f64_repr(f64::NEG_INFINITY..=self.end)
    }
}

// Integer exponent (e.g. `ldexp`) → value-uniform.
impl Draw<i32> for core::ops::Range<i32> {
    fn draw(self) -> i32 {
        rand::random_range(self)
    }
}
impl Draw<i32> for core::ops::RangeInclusive<i32> {
    fn draw(self) -> i32 {
        rand::random_range(self)
    }
}

/// Draw one input from `range`; see [`Draw`] for how the range form picks the
/// distribution.
pub fn draw<T, R: Draw<T>>(range: R) -> T {
    range.draw()
}

/// Time `f` (which draws fresh input each call) under Criterion as `name`.
pub fn run<T>(criterion: &mut criterion::Criterion, name: &str, mut f: impl FnMut() -> T) {
    criterion.bench_function(name, |bencher| bencher.iter(&mut f));
}

/// Define a Criterion bench function `$name` timing `$callback` on random input.
///
/// Each argument is a range drawn fresh per iteration; the range *form* picks
/// the distribution (bounded = value-uniform, open-ended = representation-
/// uniform — see [`Draw`]).  For example `..` is every `f32`/`f64` bit pattern,
/// `0.0..` is representation-uniform over `[0, ∞]`, and `-1.0..=1.0` is
/// value-uniform.
///
/// This stays a macro only to mint a named `fn` item per bench (so
/// `criterion_group!` can list it), to derive the bench name from `$callback`
/// via `stringify!`, and to accept a variable number of input ranges —
/// everything else is the [`draw`] and [`run`] functions.
#[macro_export]
macro_rules! bench {
    ($name:ident, $callback:expr $(, $arg:expr)* $(,)?) => {
        fn $name(criterion: &mut criterion::Criterion) {
            bench::run(criterion, stringify!($callback), || $callback($(bench::draw($arg)),*));
        }
    };
}
