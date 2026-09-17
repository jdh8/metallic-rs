//! Binary128 Gamma and log |Gamma|, evaluated with original Taylor data.
//!
//! Log Gamma on [1,2] uses its derivatives at dyadic centers, recurrence
//! multiplies the factors before taking one logarithm, and x>=64 uses the
//! Bernoulli expansion with its first-omitted-term bound (DLMF 5.11.1).
//! Reflection reduces the dyadic input exactly and evaluates sin(pi*f)/(pi*f),
//! cancelling pi symbolically before taking logarithms. In particular no
//! rounded sinpi or rounded logarithm enters the reflection formula.
//!
//! Three 64-bit limbs serve the first pass. Its explicitly propagated scale
//! gives a conservative 2^-150 bound, including cancellation at negative
//! log-Gamma zeros. A refused interval recomputes with eight limbs. The
//! accurate policy is 2^-350 relative, including the negative zeros: 512-bit
//! arithmetic leaves room for their cancellation. The corpus generator audits
//! both zeros between consecutive negative poles through -40; 57 neighborhoods
//! have nonpole nearest inputs, with minimum |log Gamma| = 2^-111.5099. Those
//! nearest inputs and neighbors also force the accurate tier below. This is a
//! precision policy rather than an exhaustive binary128 proof. Integer Gamma
//! arguments use exactly rounded integer factorials, including true midpoints.

use super::exp;
use super::gamma_tables::{self as table, Constant};
use super::{BIAS, EXP_MASK, EXP_SHIFT, QUIET_BIT, SIGN_MASK, split};

const ONE_BITS: u128 = (BIAS as u128) << EXP_SHIFT;
const TINY_BITS: u128 = ((BIAS - 256) as u128) << EXP_SHIFT;

/// Gamma, rounded to nearest, ties to even.
///
/// Signed zero returns signed infinity; negative integers and negative
/// infinity return NaN. Positive infinity returns positive infinity.
#[must_use]
pub fn tgammaq(x: f128) -> f128 {
    let b = x.to_bits();
    let a = b & !SIGN_MASK;
    if a > EXP_MASK {
        return f128::from_bits(b | QUIET_BIT);
    }
    if a == 0 {
        return f128::from_bits(EXP_MASK | (b & SIGN_MASK));
    }
    if a == EXP_MASK {
        return if b & SIGN_MASK == 0 { x } else { f128::NAN };
    }
    if let Some(n) = integer(a) {
        if b & SIGN_MASK != 0 {
            return f128::NAN;
        }
        return table::FACTORIAL
            .get(n.wrapping_sub(1))
            .map_or(f128::INFINITY, |&v| f128::from_bits(v));
    }
    // Gamma(1+x)/x = (1-EulerGamma*x+O(x²))/x. Below 2^-256
    // the correction cannot cross a reciprocal's binary128 midpoint: for
    // a 113-bit dyadic input, a nonzero reciprocal-to-midpoint distance is
    // at least 2^-226 relative. Powers of two are exact reciprocals.
    if a < TINY_BITS {
        return F::<8>::from_bits(b).recip().round();
    }
    // Noninteger negatives are smaller than 2^112. Beyond -2048 their
    // distance from a pole is >=2^-101; reflection is safely below 2^-16496.
    if a >= 2048.0_f128.to_bits() {
        return if b & SIGN_MASK == 0 {
            f128::INFINITY
        } else {
            f128::from_bits(if negative_gamma(a) { SIGN_MASK } else { 0 })
        };
    }
    let (l, _) = logarithmic::<3>(b);
    let result = exponential(l);
    let sign = b & SIGN_MASK != 0 && negative_gamma(a);
    let result = result.signed(sign);
    // |log Gamma|<2^15 in this finite-result band. The exponential amplifies
    // the logarithm's first-pass error by at most 2^15; 2^-150 relative
    // includes that amplification and the exponential's arithmetic.
    if let Some(y) = decided(result, result.e - 150) {
        return y;
    }
    tgamma_accurate(b, sign)
}

/// Natural logarithm of the absolute value of Gamma.
///
/// Poles and either infinity return positive infinity; lgamma(1) and
/// lgamma(2) are positive zero. This function does not set a signgam global.
#[must_use]
pub fn lgammaq(x: f128) -> f128 {
    let b = x.to_bits();
    let a = b & !SIGN_MASK;
    if a > EXP_MASK {
        return f128::from_bits(b | QUIET_BIT);
    }
    if a == 0 || a == EXP_MASK || (b & SIGN_MASK != 0 && integer(a).is_some()) {
        return f128::INFINITY;
    }
    if b == ONE_BITS || b == 2.0_f128.to_bits() {
        return 0.0;
    }
    let (r, scale) = logarithmic::<3>(b);
    decided(r, scale - 150).unwrap_or_else(|| lgamma_accurate(b))
}

#[cold]
#[inline(never)]
fn tgamma_accurate(bits: u128, sign: bool) -> f128 {
    let (l, _) = logarithmic::<8>(bits);
    exponential(l).signed(sign).round()
}

#[cold]
#[inline(never)]
fn lgamma_accurate(bits: u128) -> f128 {
    logarithmic::<8>(bits).0.round()
}

fn integer(a: u128) -> Option<usize> {
    let (m, e) = split(a);
    if e < 0 {
        return None;
    }
    if e >= 112 {
        return Some(usize::MAX);
    }
    let shift = (112 - e) as u32;
    if m & ((1_u128 << shift) - 1) != 0 {
        return None;
    }
    Some((m >> shift).min(usize::MAX as u128) as usize)
}

fn negative_gamma(a: u128) -> bool {
    let (m, e) = split(a);
    e < 0 || ((m >> (112 - e)) & 1) == 0
}

/// A small integer floating frame. The magnitude is normalized, with value
/// m*2^(e-64N); operations truncate below the stored limbs. Keeping a floating
/// exponent preserves tiny derivatives and avoids the positive lgamma zeros.
#[derive(Clone, Copy, Debug)]
struct F<const N: usize> {
    m: [u64; N],
    e: i32,
    neg: bool,
}

impl<const N: usize> F<N> {
    const ZERO: Self = Self {
        m: [0; N],
        e: 0,
        neg: false,
    };
    fn one() -> Self {
        Self::uint(1)
    }
    fn uint(x: u64) -> Self {
        let mut m = [0; N];
        m[N - 1] = x;
        Self {
            m,
            e: 64,
            neg: false,
        }
        .normalize()
    }
    fn from_bits(b: u128) -> Self {
        let a = b & !SIGN_MASK;
        if a == 0 {
            return Self::ZERO;
        }
        let (m, e) = split(a);
        let mut out = [0; N];
        let m = m << 15;
        out[N - 1] = (m >> 64) as u64;
        out[N - 2] = m as u64;
        Self {
            m: out,
            e: e + 1,
            neg: b & SIGN_MASK != 0,
        }
    }
    fn constant(c: &Constant) -> Self {
        let mut m = [0; N];
        m.copy_from_slice(&c.2[8 - N..]);
        Self {
            m,
            e: c.1,
            neg: c.0,
        }
    }
    fn zero(self) -> bool {
        self.m[N - 1] == 0
    }
    fn signed(mut self, negative: bool) -> Self {
        self.neg = negative && !self.zero();
        self
    }
    fn neg(mut self) -> Self {
        if !self.zero() {
            self.neg = !self.neg;
        }
        self
    }
    fn scale(mut self, k: i32) -> Self {
        self.e += k;
        self
    }
    fn normalize(mut self) -> Self {
        let Some(i) = self.m.iter().rposition(|&v| v != 0) else {
            return Self::ZERO;
        };
        let lz = (N - 1 - i) * 64 + self.m[i].leading_zeros() as usize;
        self.m = shl(self.m, lz as u32);
        self.e -= lz as i32;
        self
    }
    fn add(self, b: Self) -> Self {
        if self.zero() {
            return b;
        }
        if b.zero() {
            return self;
        }
        let (a, b) = if self.e >= b.e { (self, b) } else { (b, self) };
        let bm = shr(b.m, (a.e - b.e) as u32);
        if a.neg == b.neg {
            let (mut m, carry) = add(a.m, bm);
            let mut e = a.e;
            if carry {
                m = shr(m, 1);
                m[N - 1] |= 1 << 63;
                e += 1;
            }
            Self { m, e, neg: a.neg }.normalize()
        } else {
            let order = a.m.iter().rev().cmp(bm.iter().rev());
            if order.is_ge() {
                Self {
                    m: sub(a.m, bm),
                    e: a.e,
                    neg: a.neg,
                }
                .normalize()
            } else {
                Self {
                    m: sub(bm, a.m),
                    e: a.e,
                    neg: b.neg,
                }
                .normalize()
            }
        }
    }
    fn sub(self, b: Self) -> Self {
        self.add(b.neg())
    }
    fn mul(self, b: Self) -> Self {
        if self.zero() || b.zero() {
            return Self::ZERO;
        }
        let mut p = [0_u64; 16];
        for i in 0..N {
            let mut carry = 0_u128;
            for j in 0..N {
                let v = u128::from(self.m[i]) * u128::from(b.m[j]) + u128::from(p[i + j]) + carry;
                p[i + j] = v as u64;
                carry = v >> 64;
            }
            p[i + N] = carry as u64;
        }
        let mut m = [0; N];
        m.copy_from_slice(&p[N..2 * N]);
        let mut e = self.e + b.e;
        // Retain the bit below the high product when normalizing it.
        if m[N - 1] >> 63 == 0 {
            m = shl(m, 1);
            m[0] |= p[N - 1] >> 63;
            e -= 1;
        }
        Self {
            m,
            e,
            neg: self.neg ^ b.neg,
        }
    }
    fn div_small(self, d: u64) -> Self {
        let mut m = [0; N];
        let mut r = 0_u128;
        for i in (0..N).rev() {
            let v = (r << 64) | u128::from(self.m[i]);
            m[i] = (v / u128::from(d)) as u64;
            r = v % u128::from(d);
        }
        Self {
            m,
            e: self.e,
            neg: self.neg,
        }
        .normalize()
    }
    fn recip(self) -> Self {
        let a = self.signed(false);
        let seed = 1.0 / (a.m[N - 1] as f64 * (1.0 / 18446744073709551616.0));
        let mut q = Self::from_bits((seed as f128).to_bits()).scale(-a.e);
        for _ in 0..if N == 3 { 2 } else { 4 } {
            q = q.add(q.mul(Self::one().sub(a.mul(q))));
        }
        q.signed(self.neg)
    }
    fn to_f64(self) -> f64 {
        let a = self.m[N - 1] as f64 * crate::exp2i(i64::from(self.e - 64));
        if self.neg { -a } else { a }
    }
    fn round(self) -> f128 {
        if self.zero() {
            return 0.0;
        }
        let high = (u128::from(self.m[N - 1]) << 64) | u128::from(self.m[N - 2]);
        let sticky = u128::from(self.m[..N - 2].iter().any(|&x| x != 0));
        let result = exp::round(self.e - 1, high, sticky);
        f128::from_bits(result.to_bits() | if self.neg { SIGN_MASK } else { 0 })
    }
}

fn shl<const N: usize>(a: [u64; N], s: u32) -> [u64; N] {
    let mut r = [0; N];
    let w = (s / 64) as usize;
    let b = s % 64;
    for i in w..N {
        r[i] = a[i - w] << b;
        if b != 0 && i > w {
            r[i] |= a[i - w - 1] >> (64 - b);
        }
    }
    r
}
fn shr<const N: usize>(a: [u64; N], s: u32) -> [u64; N] {
    let mut r = [0; N];
    let w = (s / 64) as usize;
    let b = s % 64;
    for i in w..N {
        r[i - w] = a[i] >> b;
        if b != 0 && i + 1 < N {
            r[i - w] |= a[i + 1] << (64 - b);
        }
    }
    r
}
fn add<const N: usize>(a: [u64; N], b: [u64; N]) -> ([u64; N], bool) {
    let mut r = [0; N];
    let mut c = 0_u128;
    for i in 0..N {
        let v = u128::from(a[i]) + u128::from(b[i]) + c;
        r[i] = v as u64;
        c = v >> 64;
    }
    (r, c != 0)
}
fn sub<const N: usize>(a: [u64; N], b: [u64; N]) -> [u64; N] {
    let mut r = [0; N];
    let mut c = 0_u128;
    for i in 0..N {
        let v = (u128::from(a[i]) | (1 << 64)) - u128::from(b[i]) - c;
        r[i] = v as u64;
        c = u128::from(v >> 64 == 0);
    }
    r
}

fn decided(r: F<3>, error_exponent: i32) -> Option<f128> {
    let bound = F::one().scale(error_exponent);
    let a = r.sub(bound).round();
    let b = r.add(bound).round();
    (a.to_bits() == b.to_bits()).then_some(a)
}

/// Natural logarithm: m=c*(1+z), c=j/64, |z|<=1/128.
/// The exact subtraction is followed by division by the small integer j.
fn logarithm<const N: usize>(a: F<N>) -> F<N> {
    let e = a.e - 1;
    let m = F { e: 1, ..a };
    let j = ((m.m[N - 1] >> 56) + 1) / 2; // nearest j in 64..128
    let c = F::uint(j).scale(-6);
    let z = m.sub(c).scale(6).div_small(j);
    // The first omitted term is below 2^-200 / 2^-552 respectively.
    let terms = if N == 3 { 27 } else { 78 };
    let c = &table::LOG_COEF[..terms];
    let mut p = F::constant(&c[terms - 1]);
    for c in c[..terms - 1].iter().rev() {
        p = F::constant(c).sub(z.mul(p));
    }
    let sum = z.mul(p);
    let base = F::constant(&table::LOG[(j - 64) as usize]);
    let offset = F::constant(&table::LN2)
        .mul(F::uint(e.unsigned_abs() as u64))
        .signed(e < 0);
    // LOG[64] and LN2 are the same constant. Cancel those terms exactly
    // before folding a tiny residual near one.
    offset.add(base).add(sum)
}

fn exponential<const N: usize>(a: F<N>) -> F<N> {
    if a.zero() {
        return F::one();
    }
    // Saturate outside the final binary128 range, avoiding a wide integer
    // conversion for reflected inputs well past the underflow threshold.
    if a.e > 15 {
        return F::one().scale(if a.neg { -20000 } else { 20000 });
    }
    let z = a.mul(F::constant(&table::LOG2E));
    let mut n = z.to_f64().floor() as i32;
    let integer = |n: i32| F::uint(n.unsigned_abs() as u64).signed(n < 0);
    let mut f = z.sub(integer(n));
    if f.neg {
        n -= 1;
        f = f.add(F::one());
    }
    if !f.zero() && f.e > 0 {
        n += 1;
        f = f.sub(F::one());
    }
    let j = (f.to_f64() * 64.0).floor().clamp(0.0, 63.0) as usize;
    let r = f
        .sub(F::uint(j as u64).scale(-6))
        .mul(F::constant(&table::LN2));
    // |r|<=ln(2)/64: the omitted terms are below 2^-202 / 2^-528.
    let terms = if N == 3 { 20 } else { 48 };
    let mut sum = F::constant(&table::EXP_COEF[terms]);
    for c in table::EXP_COEF[..terms].iter().rev() {
        sum = F::constant(c).add(r.mul(sum));
    }
    sum.mul(F::constant(&table::EXP2[j])).scale(n)
}

/// log Gamma at a dyadic Taylor center. Keeping the zero constant at c=1
/// and c=2 exact makes the leading derivative relative, however small h is.
fn local<const N: usize>(x: F<N>) -> F<N> {
    // These integer-mantissa kernels cannot use the machine-float poly helper.
    let j = ((x.to_f64() - 1.0) * 16.0).round().clamp(0.0, 16.0) as usize;
    let h = x.sub(F::uint(16 + j as u64).scale(-4));
    // |h|<=1/32. The first omitted Taylor term is below 2^-190 / 2^-530.
    let count = if N == 3 { 37 } else { 105 };
    let c = &table::LGAMMA[j];
    let mut p = F::constant(&c[count - 1]);
    for v in c[..count - 1].iter().rev() {
        p = F::constant(v).add(h.mul(p));
    }
    p
}

/// Raw positive log Gamma, plus an exponent bounding its uncancelled terms.
fn positive<const N: usize>(x: F<N>) -> (F<N>, i32) {
    // The recurrence below one subtracts two small logarithms. Take the
    // Taylor series directly near its zero to preserve relative accuracy.
    if x.e == 0 && x.to_f64() >= 0.96875 {
        let r = local(x);
        return (r, r.e);
    }
    if x.e <= 0 {
        let l = logarithm(x);
        let r = local(F::one().add(x)).sub(l);
        return (r, r.e.max(l.e));
    }
    if x.e <= 2 && x.to_f64() <= 2.0 {
        let r = local(x);
        return (r, r.e);
    }
    if x.e <= 6 {
        let n = (x.to_f64().floor() as u64).saturating_sub(1);
        let y = x.sub(F::uint(n));
        let mut p = F::one();
        for k in 0..n {
            p = p.mul(y.add(F::uint(k)));
        }
        let l = logarithm(p);
        let r = l.add(local(y));
        return (r, r.e.max(l.e));
    }
    let l = logarithm(x);
    let main = x.sub(F::one().scale(-1)).mul(l).sub(x);
    let inv = x.recip();
    let u = inv.mul(inv);
    // At x=64 the first omitted terms are below 2^-197 / 2^-517;
    // for positive real x each bounds the entire Stirling remainder.
    let count = if N == 3 { 20 } else { 112 };
    let mut p = F::constant(&table::STIRLING[count - 1]);
    for c in table::STIRLING[..count - 1].iter().rev() {
        p = F::constant(c).add(u.mul(p));
    }
    let r = main.add(F::constant(&table::HALF_LOG_TAU)).add(inv.mul(p));
    (r, r.e.max(main.e))
}

/// Exact distance to the nearest integer, for a nonintegral positive input.
fn fraction<const N: usize>(bits: u128) -> F<N> {
    let (m, e) = split(bits);
    if e < -1 {
        return F::from_bits(bits);
    }
    let shift = (112 - e) as u32;
    let one = 1_u128 << shift;
    let f = m & (one - 1);
    let f = f.min(one - f);
    let mut r = F::ZERO;
    r.m[N - 1] = (f >> 64) as u64;
    r.m[N - 2] = f as u64;
    r.e = e + 16;
    r.normalize()
}

/// log(sin(pi*f)/(pi*f)), 0<f<=1/2. The sinc series remains close to one,
/// so its absolute error is also a relative error in the reflection formula.
fn log_sinc<const N: usize>(f: F<N>) -> F<N> {
    if f.e < -(N as i32 * 32 + 8) {
        return F::ZERO;
    }
    let t = f.mul(F::constant(&table::PI));
    let u = t.mul(t).neg();
    // The alternating remainder is below 2^-197 / 2^-527 at |pi*f|<=pi/2.
    let terms = if N == 3 { 25 } else { 54 };
    let mut sum = F::constant(&table::SINC_COEF[terms]);
    for c in table::SINC_COEF[..terms].iter().rev() {
        sum = F::constant(c).add(u.mul(sum));
    }
    logarithm(sum)
}

fn logarithmic<const N: usize>(bits: u128) -> (F<N>, i32) {
    let a = F::from_bits(bits & !SIGN_MASK);
    if bits & SIGN_MASK == 0 {
        return positive(a);
    }
    if a.e <= -1 {
        let l = logarithm(a);
        let r = local(F::uint(2).sub(a))
            .sub(logarithm(F::one().sub(a)))
            .sub(l);
        return (r, r.e.max(l.e));
    }
    let (g, scale) = positive(a);
    let l = logarithm(a);
    let f = fraction(bits & !SIGN_MASK);
    let lf = logarithm(f);
    let ls = log_sinc(f);
    let r = g.add(l).add(lf).add(ls).neg();
    (r, scale.max(l.e).max(lf.e).max(ls.e))
}

#[cfg(all(test, feature = "mpfr"))]
mod ziv_soundness {
    use super::super::MANTISSA_MASK;
    use super::*;
    use rug::{Float, float::Round::Nearest, ops::Pow};
    const PREC: u32 = 640;

    /// Nearest binary128 inputs to the negative log-Gamma zeros with nonpole
    /// nearest inputs. Both roots in every interval (-n-1,-n), n=2..40,
    /// were solved independently with MPFR512, then rounded to binary128.
    /// Generate: `CC=clang cargo +nightly run --release --features "f128 mpfr" --example gen_f128_gamma_cases -- audit-roots /tmp/metallic-gamma-negative-roots.txt`.
    const NEGATIVE_ROOTS: [u128; 57] = [
        0xc0003a7fc9600f86_c155f64f98af8d04,
        0xc0005fb410a1bd90_0cbccad25a320f57,
        0xc0009260dbc9e59a_f7dc5f34cd69ecdd,
        0xc000fa471547c2fe_50b86a2b094891b6,
        0xc0010284e7859958_0c30e7c2c3786045,
        0xc0013f7577a6eeaf_cd44342a901da621,
        0xc0014086a57f0b6d_90ca9315b9654e53,
        0xc0017fe92f591f40_d5f753b58b2f4c7b,
        0xc0018016b25897c8_ced81f0b65b458e1,
        0xc001bffcbf76b86e_ffcf589ad9708ff5,
        0xc001c0033fdedfe1_eb7d120b736e61e1,
        0xc001ffff97f8159c_f0f2a7a0ad48ac33,
        0xc002000034028b3f_93ec19679d839d91,
        0xc0021ffffa3884bd_01ff90c9d2ae924b,
        0xc002200005c7768f_b0dadb087fdb86a4,
        0xc0023fffff6c0d7b_fb9a0c55cef4a0a9,
        0xc00240000093f277_7324f68bb2bc2a88,
        0xc0025ffffff28cdd_3e366db59a55b7a3,
        0xc0026000000d7322_a62bb2cb4dffd482,
        0xc0027ffffffee112_70e70fbc835987a8,
        0xc002800000011eed_8ee62acf81e04a1e,
        0xc0029fffffffe9ed_b9ec2ff2f4bff8eb,
        0xc002a00000001612_461380cd07ba3720,
        0xc002bffffffffe6c_68b573c30947c13a,
        0xc002c00000000193_974a8bd29cebd8ae,
        0xc002dfffffffffe5_180c18cc09de606e,
        0xc002e0000000001a_e7f3e7337a1c841f,
        0xc002fffffffffffe_5180c18cc43ea25d,
        0xc003000000000000_d73f9f399da1424c,
        0xc0030fffffffffff_f3569c47e7a93e1c,
        0xc003100000000000_0ca963b818568887,
        0xc0031fffffffffff_ff4bec3ce234132d,
        0xc003200000000000_00b413c31dcbeca5,
        0xc0032fffffffffff_fff685b25cbf5f54,
        0xc003300000000000_00097a4da340a0ac,
        0xc0033fffffffffff_ffff86af516ff7f7,
        0xc003400000000000_00007950ae900809,
        0xc0034fffffffffff_fffffa391c4248c3,
        0xc003500000000000_000005c6e3bdb73d,
        0xc0035fffffffffff_ffffffbcc71a4920,
        0xc003600000000000_0000004338e5b6e0,
        0xc0036fffffffffff_fffffffd13c97d9d,
        0xc003700000000000_00000002ec368263,
        0xc0037fffffffffff_ffffffffe0d30fe7,
        0xc003800000000000_000000001f2cf019,
        0xc0038fffffffffff_fffffffffec0c332,
        0xc003900000000000_00000000013f3cce,
        0xc0039fffffffffff_fffffffffff3b8bd,
        0xc003a00000000000_00000000000c4743,
        0xc003afffffffffff_ffffffffffff8b95,
        0xc003b00000000000_000000000000746b,
        0xc003bfffffffffff_fffffffffffffbd8,
        0xc003c00000000000_0000000000000428,
        0xc003cfffffffffff_ffffffffffffffdb,
        0xc003d00000000000_0000000000000025,
        0xc003dfffffffffff_ffffffffffffffff,
        0xc003e00000000000_0000000000000001,
    ];

    fn value<const N: usize>(v: F<N>) -> Float {
        let mut r = Float::with_val(PREC, 0);
        for &m in v.m.iter().rev() {
            r <<= 64;
            r += m;
        }
        r *= Float::with_val(PREC, 2).pow(v.e - 64 * N as i32);
        if v.neg { -r } else { r }
    }
    fn mix(mut x: u64) -> u64 {
        x = x.wrapping_mul(0x2545_f491_4f6c_dd1d);
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        x ^ (x >> 31)
    }
    fn draw(i: u64) -> f128 {
        if i % 257 == 0 {
            let root = NEGATIVE_ROOTS[(i / 257 % 57) as usize];
            return f128::from_bits(root.wrapping_add_signed((i / (257 * 57) % 5) as i128 - 2));
        }
        let b = u128::from(mix(i)) | (u128::from(mix(i ^ 0xabcd)) << 64);
        let m = b & MANTISSA_MASK;
        let x = match i % 8 {
            0 => f128::from_bits((((i / 8) % 0x7fff) as u128) << 112 | m | 1),
            1 => f128::from_bits(((BIAS - 256 + (i / 8 % 269) as i32) as u128) << 112 | m),
            2 => f128::from_bits(ONE_BITS | m),
            3 => f128::from_bits(((BIAS - 1) as u128) << 112 | m),
            4 => f128::from_bits(((BIAS + 6 + (i / 8 % 8) as i32) as u128) << 112 | m),
            5 => f128::from_bits(
                (ONE_BITS + (i / 8 % 65) as u128 * ((1_u128 << 112) / 16))
                    .wrapping_add_signed((b % 17) as i128 - 8),
            ),
            6 => f128::from_bits((2.0_f128.to_bits()).wrapping_add_signed((b % 257) as i128 - 128)),
            _ => f128::from_bits(((BIAS + (i / 8 % 6) as i32) as u128) << 112 | m),
        };
        if i % 3 == 0 && x < 2048.0 { -x } else { x }
    }
    fn truth(x: f128, gamma: bool) -> Float {
        let mut y = Float::with_val(PREC, x);
        if gamma {
            y.gamma_mut();
        } else {
            y.ln_abs_gamma_mut();
        }
        y
    }
    fn check<const N: usize>(x: f128, gamma: bool) -> F<N> {
        let l = logarithmic::<N>(x.to_bits()).0;
        if gamma {
            exponential(l).signed(x < 0.0 && negative_gamma(x.to_bits() & !SIGN_MASK))
        } else {
            l
        }
    }
    #[test]
    fn arithmetic() {
        for i in 1..20_000_u64 {
            let a = F::<8>::from_bits(draw(i).abs().to_bits());
            let b = F::<8>::from_bits(draw(i ^ 0xcafe).abs().to_bits());
            let va = value(a);
            let vb = value(b);
            for (r, want) in [
                (a.add(b), Float::with_val(PREC, &va + &vb)),
                (a.sub(b), Float::with_val(PREC, &va - &vb)),
                (a.mul(b), Float::with_val(PREC, &va * &vb)),
                (a.recip(), Float::with_val(PREC, 1) / &va),
            ] {
                if !want.is_zero() {
                    let error = ((value(r) - &want) / &want).abs();
                    assert!(
                        error < Float::with_val(PREC, 2).pow(-495),
                        "i={i} err={error}"
                    );
                }
            }
        }
    }
    #[test]
    fn gamma_quick_mpfr() {
        for i in 1..20_000_u64 {
            let x = draw(i);
            for gamma in [false, true] {
                if x < 0.0 && integer(x.abs().to_bits()).is_some() {
                    continue;
                }
                let want = super::super::mpfr::cr_unop(x, |y| {
                    if gamma {
                        y.gamma_round(Nearest)
                    } else {
                        y.ln_abs_gamma_round(Nearest).1
                    }
                });
                let got = if gamma { tgammaq(x) } else { lgammaq(x) };
                assert_eq!(
                    got.to_bits(),
                    want.to_bits(),
                    "gamma={gamma}, x={x:?}, bits={:032x}, got={got:?}, want={want:?}",
                    x.to_bits()
                );
            }
        }
    }
    fn certify(gamma: bool) {
        let mut worst = 0.0_f64;
        let mut at = 0.0_f128;
        let mut wide_worst = 0.0_f64;
        let mut refused = 0;
        for i in 1..=1_000_000_u64 {
            let x = draw(i);
            let a = x.to_bits() & !SIGN_MASK;
            if (integer(a).is_some() && (gamma || x < 0.0))
                || (gamma && (!(TINY_BITS..2048.0_f128.to_bits()).contains(&a)))
            {
                continue;
            }
            let (l, scale) = logarithmic::<3>(x.to_bits());
            let r = if gamma {
                exponential(l).signed(x < 0.0 && negative_gamma(a))
            } else {
                l
            };
            let bound = if gamma { r.e - 150 } else { scale - 150 };
            let want = truth(x, gamma);
            if !want.is_finite() || want.is_zero() {
                continue;
            }
            // Saturated gamma results have already crossed the binary128 range.
            if gamma && (r.e >= 20000 || r.e <= -20000) {
                continue;
            }
            let ratio = ((value(r) - &want).abs() * Float::with_val(PREC, 2).pow(-bound)).to_f64();
            if ratio > worst {
                worst = ratio;
                at = x;
            }
            let got = decided(r, bound);
            refused += u64::from(got.is_none());
            if let Some(got) = got {
                let reference = super::super::mpfr::cr_unop(x, |y| {
                    if gamma {
                        y.gamma_round(Nearest)
                    } else {
                        y.ln_abs_gamma_round(Nearest).1
                    }
                });
                assert_eq!(got.to_bits(), reference.to_bits(), "gamma={gamma} x={x:?}");
            }
            if i % 1021 == 0 || got.is_none() {
                let wide = check::<8>(x, gamma);
                let ratio: Float =
                    ((value(wide) - &want) / &want).abs() * Float::with_val(PREC, 2).pow(350);
                wide_worst = wide_worst.max(ratio.to_f64());
                assert!(ratio < 0.5, "wide gamma={gamma} x={x:?} ratio={ratio}");
            }
        }
        println!(
            "gamma={gamma}: worst |err|/gate={worst:e} at {at:?}; accurate relative err/2^-350={wide_worst:e}; refused={refused}"
        );
        assert!(worst < 0.5);
    }
    #[test]
    fn lgammaq_fast_leg_is_sound() {
        certify(false);
    }
    #[test]
    fn tgammaq_fast_leg_is_sound() {
        certify(true);
    }

    #[test]
    fn accurate_negative_roots() {
        let mut worst = 0.0_f64;
        let mut smallest = f64::INFINITY;
        for root in NEGATIVE_ROOTS {
            for delta in -4..=4_i128 {
                let bits = root.wrapping_add_signed(delta);
                if integer(bits & !SIGN_MASK).is_some() {
                    continue;
                }
                let x = f128::from_bits(bits);
                let want = truth(x, false);
                smallest = smallest.min(want.clone().abs().to_f64());
                let r = check::<8>(x, false);
                let error: Float =
                    ((value(r) - &want) / &want).abs() * Float::with_val(PREC, 2).pow(350);
                worst = worst.max(error.to_f64());
                let rounded = super::super::mpfr::cr_unop(x, |y| y.ln_abs_gamma_round(Nearest).1);
                assert_eq!(
                    r.round().to_bits(),
                    rounded.to_bits(),
                    "forced root x={x:?}"
                );
                assert_eq!(
                    lgammaq(x).to_bits(),
                    rounded.to_bits(),
                    "public root x={x:?}"
                );
            }
        }
        println!(
            "negative lgamma root audit: min |lgamma|={smallest:e}; accurate relative err/2^-350={worst:e}"
        );
        assert!(worst < 0.5);
    }

    #[test]
    fn accurate_edges() {
        for x in (0..=128).map(|i| i as f128 / 16.0).chain([
            31.0 / 32.0,
            64.0,
            1755.5,
            2048.0,
            f128::from_bits(TINY_BITS),
        ]) {
            let anchor = x.to_bits();
            for d in -3..=3_i128 {
                let a = anchor.wrapping_add_signed(d);
                if a == 0 || a >= EXP_MASK {
                    continue;
                }
                for negative in [false, true] {
                    let x = f128::from_bits(a | if negative { SIGN_MASK } else { 0 });
                    if negative && integer(a).is_some() {
                        continue;
                    }
                    for gamma in [false, true] {
                        if gamma && integer(a).is_some() {
                            continue;
                        }
                        let want = super::super::mpfr::cr_unop(x, |y| {
                            if gamma {
                                y.gamma_round(Nearest)
                            } else {
                                y.ln_abs_gamma_round(Nearest).1
                            }
                        });
                        let got = check::<8>(x, gamma).round();
                        assert_eq!(got.to_bits(), want.to_bits(), "wide gamma={gamma} x={x:?}");
                    }
                }
            }
        }
    }
}
