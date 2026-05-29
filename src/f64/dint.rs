//! Accurate-path 128-bit fixed-point type for the correctly-rounded logarithm.
//!
//! This is an idiomatic Rust port of Tom Hubrecht's `dint64_t` arithmetic from
//! CORE-MATH (`binary64/log/dint.h` and `log.c`).  A [`Dint`] represents the
//! value `(-1)^sgn · (m / 2^127) · 2^ex`, with the 128-bit significand `m`
//! normalized so that bit 127 is set (`m ∈ [2^127, 2^128)`) for nonzero values.
//! CORE-MATH stores the significand as two `u64` words `{hi, lo}`; here they are
//! packed as `m = (hi << 64) | lo`.

use super::dint_consts::{INVERSE_2, LOG2, LOG_INV_2, M_ONE, P_2, ZERO};

/// 128-bit fixed-point number used by the accurate logarithm path.
///
/// Value = `(-1)^sgn · (m / 2^127) · 2^ex`.  For nonzero values `m` is
/// normalized with bit 127 set.
#[derive(Debug, Clone, Copy)]
pub struct Dint {
    /// Sign bit: `true` is negative.
    pub sgn: bool,

    /// Binary exponent.
    pub ex: i64,

    /// 128-bit significand, normalized so bit 127 is set for nonzero values.
    pub m: u128,
}

impl Dint {
    /// Compare magnitudes (ignoring sign): like `cmp_dint` in CORE-MATH.
    #[inline]
    fn cmp_magnitude(&self, other: &Self) -> core::cmp::Ordering {
        self.ex.cmp(&other.ex).then_with(|| self.m.cmp(&other.m))
    }

    /// Round-to-nearest sum of two `Dint`s (port of `add_dint`).
    ///
    /// Handles full normalization, including equal-magnitude opposite-sign
    /// cancellation to [`ZERO`] and the same-sign carry on overflow.
    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        if self.m == 0 {
            return *other;
        }
        if other.m == 0 {
            return *self;
        }

        // Ensure |self| >= |other| so the rest assumes A >= B in magnitude.
        let (a, b) = match self.cmp_magnitude(other) {
            core::cmp::Ordering::Equal => {
                if self.sgn != other.sgn {
                    return ZERO;
                }
                // Equal magnitudes, same sign: result is 2·a.
                return Self {
                    sgn: self.sgn,
                    ex: self.ex + 1,
                    m: self.m,
                };
            }
            core::cmp::Ordering::Less => (other, self),
            core::cmp::Ordering::Greater => (self, other),
        };

        // From now on |a| > |b|.
        let big = a.m;
        let mut small = b.m;
        let mut m_ex = a.ex;

        if a.ex > b.ex {
            let sh = a.ex - b.ex;
            // Round to nearest before discarding the shifted-out bits.
            if sh <= 128 {
                small = small.wrapping_add(1 & (small >> (sh - 1)));
            }
            small = if sh < 128 { small >> sh } else { 0 };
        }

        let sgn = a.sgn;
        let mut c: u128;

        if a.sgn != b.sgn {
            // Different signs: C = A - B (no borrow since |A| > |B|).
            c = big.wrapping_sub(small);
        } else {
            let (sum, carry) = big.overflowing_add(small);
            c = sum;
            if carry {
                // Carry out of bit 127: shift right one, rounding, restore bit 127.
                c = c.wrapping_add(c & 1);
                c = (1u128 << 127) | (c >> 1);
                m_ex += 1;
            }
        }

        // Renormalize so bit 127 is set.
        let shift = c.leading_zeros();
        c <<= shift;

        Self {
            sgn,
            ex: m_ex - i64::from(shift),
            m: c,
        }
    }

    /// Round-to-nearest product of two `Dint`s with 126-bit accuracy (port of
    /// `mul_dint`).
    ///
    /// Rust has no `u256`, so the `128×128 → 256` product is formed from
    /// `u64×u64 → u128` partial products, mirroring CORE-MATH's `{hi, lo}`
    /// arithmetic.  `t` holds the top 128 bits of the product; `m.l` (the bit
    /// just below) is the round bit.
    #[inline]
    pub fn mul(&self, other: &Self) -> Self {
        let a_hi = (self.m >> 64) as u64;
        let a_lo = self.m as u64;
        let b_hi = (other.m >> 64) as u64;
        let b_lo = other.m as u64;

        // t = a_hi · b_hi (top 128 bits of the product so far)
        let mut t = u128::from(a_hi) * u128::from(b_hi);

        let m1 = u128::from(a_hi) * u128::from(b_lo);
        let m2 = u128::from(a_lo) * u128::from(b_hi);

        // m = m1 + m2, carry folds into t's high word (i.e. t += carry << 64).
        let (m, carry) = m1.overflowing_add(m2);
        if carry {
            t = t.wrapping_add(1u128 << 64);
        }
        // t += high 64 bits of m
        t = t.wrapping_add(u128::from((m >> 64) as u64));

        // Ensure the leading bit (bit 127 of t) is set.
        let ex = u32::from((t >> 127) == 0);
        if ex == 1 {
            t <<= 1;
        }

        // Round bit: bit 63 of the low word of m.
        t = t.wrapping_add(u128::from((m as u64) >> 63));

        Self {
            sgn: self.sgn ^ other.sgn,
            ex: self.ex + other.ex - i64::from(ex) + 1,
            m: t,
        }
    }

    /// Product with a signed integer (port of `mul_dint_2`).
    #[inline]
    pub fn mul_int(&self, b: i64) -> Self {
        if b == 0 {
            return ZERO;
        }

        let c = b.unsigned_abs();
        let sgn = if b < 0 { !self.sgn } else { self.sgn };

        let a_hi = (self.m >> 64) as u64;
        let a_lo = self.m as u64;

        // t = a_hi · c, then left-justify it.
        let mut t = u128::from(a_hi) * u128::from(c);
        let leading = (t >> 64) as u64;
        let m = if leading != 0 {
            leading.leading_zeros()
        } else {
            64
        };
        t <<= m;

        // l = (a_lo · c) shifted to align, keeping one rounding bit.
        let l = (u128::from(a_lo) * u128::from(c)) << (m - 1) >> 63;

        let (mut sum, carry) = l.overflowing_add(t);
        let mut m = m;
        if carry {
            sum = sum.wrapping_add(sum & 1);
            sum = (1u128 << 127) | (sum >> 1);
            m -= 1;
        }

        Self {
            sgn,
            ex: self.ex + 64 - i64::from(m),
            m: sum,
        }
    }

    /// Convert a finite positive `f64` to a `Dint` (port of `dint_fromd`,
    /// including the subnormal preparation `cr_log` performs before calling it).
    #[inline]
    pub fn from_f64(b: f64) -> Self {
        // `cr_log` scales subnormals by 2^52 and adjusts the exponent.
        let (b, bias) = if b.abs() < f64::MIN_POSITIVE {
            (b * f64::from_bits(0x4330_0000_0000_0000), 52) // 2^52
        } else {
            (b, 0)
        };

        let bits = b.to_bits();
        let biased = ((bits >> 52) & 0x7ff) as i64;
        // Significand with the implicit leading bit restored.
        let hi = (bits & (!0u64 >> 12)) + (1u64 << 52);
        let ex = biased - 0x3ff - bias;

        // Left-justify `hi` (a 53-bit value) into the top word.
        let t = hi.leading_zeros();
        Self {
            sgn: b < 0.0,
            ex: ex - i64::from(if t > 11 { t - 12 } else { 0 }),
            m: u128::from(hi << t) << 64,
        }
    }

    /// Convert to `f64` with correct rounding (port of `dint_tod`).
    ///
    /// Assumes the result is in the normal range, which always holds for `ln`.
    #[inline]
    #[allow(clippy::wrong_self_convention)] // `Dint` is `Copy`; `&self` avoids a move
    pub fn to_f64(&self) -> f64 {
        let hi = (self.m >> 64) as u64;

        // Upper 53 bits of `hi` as a mantissa in [1, 2).
        let mut r = f64::from_bits((hi >> 11) | (0x3ffu64 << 52));

        let mut rd = 0.0;
        // Round bit (bit 10 of hi): add 2^-53.
        if (hi >> 10) & 1 == 1 {
            rd += f64::from_bits(0x3ca0_0000_0000_0000); // 2^-53
        }
        // Sticky bits: add 2^-54.
        if hi & 0x3ff != 0 || (self.m as u64) != 0 {
            rd += f64::from_bits(0x3c90_0000_0000_0000); // 2^-54
        }

        if self.sgn {
            r = -r;
            r -= rd;
        } else {
            r += rd;
        }

        // Scale by 2^ex.  For `ln`, -1023 < ex < 1023, so this is one exact step.
        let e = f64::from_bits((((self.ex + 1023) & 0x7ff) as u64) << 52);
        r * e
    }
}

/// Degree-12 Horner evaluation of the accurate-path polynomial (port of `p_2`).
#[inline]
fn p_2(z: &Dint) -> Dint {
    let mut r = P_2[0];
    for p in &P_2[1..] {
        r = z.mul(&r);
        r = p.add(&r);
    }
    z.mul(&r)
}

/// Accurate `ln(x)` for a normalized positive `Dint` (port of `log_2`).
#[inline]
fn log_2(x: &Dint) -> Dint {
    let mut e = x.ex;
    let mut i = (x.m >> (64 + 55)) as usize; // x->hi >> 55

    // √2 split: if the leading word exceeds √2 (scaled), bump the exponent.
    if (x.m >> 64) as u64 > 0xb504_f333_f9de_6484 {
        e += 1;
        i >>= 1;
    }

    // x reduced to [1/√2, √2) by adjusting only the exponent.
    let xr = Dint {
        sgn: x.sgn,
        ex: x.ex - e,
        m: x.m,
    };

    let z = xr.mul(&INVERSE_2[i - 128]);
    let z = M_ONE.add(&z);

    let r = LOG2.mul_int(e);
    let p = p_2(&z);
    let p = LOG_INV_2[i - 128].add(&p);
    p.add(&r)
}

/// Correctly-rounded accurate-path natural logarithm.
#[inline]
pub fn ln_accurate(x: f64) -> f64 {
    if x == 1.0 {
        return 0.0;
    }
    log_2(&Dint::from_f64(x)).to_f64()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The accurate path must match the CORE-MATH oracle bit-for-bit.
    #[cfg(feature = "core-math")]
    #[test]
    fn accurate_matches_oracle() {
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };

        for _ in 0..2_000_000 {
            let bits = next();
            let x = f64::from_bits(bits);
            if !(x.is_finite() && x > 0.0) {
                continue;
            }
            let got = ln_accurate(x);
            let want = core_math::log(x);
            assert_eq!(
                got.to_bits(),
                want.to_bits(),
                "ln_accurate({x:e}) = {got:e}, oracle = {want:e}"
            );
        }

        // Dense sweep near 1, where cancellation stresses the accurate path.
        for k in -1_000_000i64..1_000_000 {
            let x = 1.0 + (k as f64) * f64::from_bits(0x3cb0_0000_0000_0000); // ~2^-52
            if x <= 0.0 {
                continue;
            }
            assert_eq!(ln_accurate(x).to_bits(), core_math::log(x).to_bits());
        }
    }
}
