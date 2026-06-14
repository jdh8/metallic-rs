//! Accurate-path 256-bit fixed-point type for the correctly-rounded power
//! function (Ziv's third iteration).
//!
//! This is an idiomatic Rust port of Tom Hubrecht's `qint64_t` arithmetic from
//! CORE-MATH (`binary64/pow/qint.h`), the 256-bit sibling of the 128-bit
//! [`Dint`](super::dint::Dint).  A [`Qint`] represents
//! `(-1)^sgn · (M / 2^255) · 2^ex`, with the 256-bit significand `M` normalized
//! so bit 255 is set (`M ∈ [2^255, 2^256)`) for nonzero values.
//!
//! CORE-MATH stores `M` as four `u64` words `{hh, hl, lh, ll}` (most to least
//! significant).  Rust has no `u256`, so here `M` is packed into two `u128`
//! words: `hi = (hh << 64) | hl` (top half) and `lo = (lh << 64) | ll` (bottom
//! half), i.e. `M = (hi << 128) | lo`.  The 128×128→256 sub-products are formed
//! from `u64×u64→u128` partials, mirroring the C limb arithmetic.
#![allow(clippy::cast_possible_truncation)]
// Faithful port of CORE-MATH's `qint.h`; keep the C's branch structure (which
// trips these style lints) so the limb arithmetic stays auditable.
#![allow(clippy::if_same_then_else, clippy::missing_const_for_fn)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::wrong_self_convention)] // `to_f64`/`to_i64` take `&self` (Copy type)

use super::pow_consts::ZERO_Q;

/// 256-bit fixed-point number used by the accurate power path.
///
/// Value = `(-1)^sgn · (((hi << 128) | lo) / 2^255) · 2^ex`.  For nonzero values
/// the significand is normalized with bit 255 (bit 127 of `hi`) set.
#[derive(Debug, Clone, Copy)]
pub struct Qint {
    /// Sign bit: `true` is negative.
    pub sgn: bool,
    /// Binary exponent.
    pub ex: i64,
    /// Top 128 bits of the significand (`hh:hl`).
    pub hi: u128,
    /// Bottom 128 bits of the significand (`lh:ll`).
    pub lo: u128,
}

/// `(hh, hl, lh, ll)` limb view of a significand (most → least significant).
#[inline]
const fn limbs(hi: u128, lo: u128) -> (u64, u64, u64, u64) {
    ((hi >> 64) as u64, hi as u64, (lo >> 64) as u64, lo as u64)
}

impl Qint {
    /// `true` iff the value is zero (top limb `hh` is zero ⇒ normalized zero).
    #[inline]
    const fn is_zero(&self) -> bool {
        (self.hi >> 64) == 0
    }

    /// Compare magnitudes by `(ex, hi, lo)` (port of `cmp_qint`).
    #[inline]
    fn cmp_full(&self, other: &Self) -> core::cmp::Ordering {
        self.ex
            .cmp(&other.ex)
            .then_with(|| self.hi.cmp(&other.hi))
            .then_with(|| self.lo.cmp(&other.lo))
    }

    /// Compare magnitudes by the upper two limbs only (port of `cmp_qint_22`).
    #[inline]
    fn cmp_22(&self, other: &Self) -> core::cmp::Ordering {
        self.ex.cmp(&other.ex).then_with(|| self.hi.cmp(&other.hi))
    }

    /// Round-to-nearest sum, full 256-bit operands (port of `add_qint`).
    ///
    /// Error ≤ 2 ulps(256); exact when Sterbenz applies (opposite signs,
    /// `|a|/2 ≤ |b| ≤ |a|`).
    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        if self.is_zero() {
            return *other;
        }
        if other.is_zero() {
            return *self;
        }

        let (a, b) = match self.cmp_full(other) {
            core::cmp::Ordering::Equal => {
                if self.sgn != other.sgn {
                    return ZERO_Q;
                }
                return Self {
                    ex: self.ex + 1,
                    ..*self
                };
            }
            core::cmp::Ordering::Less => (other, self),
            core::cmp::Ordering::Greater => (self, other),
        };

        // |a| > |b| from here, so a.ex >= b.ex.
        let mut ah = a.hi;
        let mut al = a.lo;
        let (mut bh, mut bl);
        let m_ex = a.ex;
        let k = (a.ex - b.ex) as u32;

        // Align b right by k bits across the 256-bit significand.
        if k >= 256 {
            bh = 0;
            bl = 0;
        } else if k >= 128 {
            bl = b.hi >> (k - 128);
            bh = 0;
        } else if k > 0 {
            bl = (b.lo >> k) | (b.hi << (128 - k));
            bh = b.hi >> k;
        } else {
            bh = b.hi;
            bl = b.lo;
        }

        let sgn = a.sgn;
        let mut r_ex = m_ex;
        let (ch, cl);

        if a.sgn != b.sgn {
            // Subtraction C = A - B (no underflow since |A| > |B|).
            let (mut c_hi, borrow0) = ah.overflowing_sub(bh);
            let (c_lo, b1) = al.overflowing_sub(bl);
            if b1 {
                c_hi = c_hi.wrapping_sub(1);
            }
            let _ = borrow0;

            let mut ex = clz256(c_hi, c_lo);
            // ex < 256 since |A| > |B|.
            if ex > 0 {
                // Re-form the difference from the unshifted operands, both
                // shifted left by ex (b additionally by -k), so the leading 1
                // lands in bit 255 without losing low bits.
                if ex >= 128 {
                    ah = al << (ex - 128);
                    al = 0;
                } else {
                    ah = (ah << ex) | (al >> (128 - ex));
                    al <<= ex;
                }
                let sh = ex as i64 - k as i64;
                bh = b.hi;
                bl = b.lo;
                if sh >= 0 {
                    let sh = sh as u32;
                    if sh >= 128 {
                        bh = bl << (sh - 128);
                        bl = 0;
                    } else if sh > 0 {
                        bh = (bh << sh) | (bl >> (128 - sh));
                        bl <<= sh;
                    }
                } else {
                    let j = (-sh) as u32;
                    if j >= 128 {
                        bl = bh >> (j - 128);
                        bh = 0;
                    } else {
                        bl = (bh << (128 - j)) | (bl >> j);
                        bh >>= j;
                    }
                }
                r_ex -= i64::from(ex);
                let (mut c2_hi, _) = ah.overflowing_sub(bh);
                let (c2_lo, b2) = al.overflowing_sub(bl);
                if b2 {
                    c2_hi = c2_hi.wrapping_sub(1);
                }
                c_hi = c2_hi;
                let c_lo2 = c2_lo;
                ex = clz256(c_hi, c_lo2);
                let final_norm = normalize_left(c_hi, c_lo2, ex);
                ch = final_norm.0;
                cl = final_norm.1;
                r_ex -= i64::from(ex);
            } else {
                ch = c_hi;
                cl = c_lo;
                r_ex -= i64::from(ex); // ex == 0
            }
        } else {
            // Addition C = A + B.
            let (mut sum_lo, carry_lo) = al.overflowing_add(bl);
            let (mut sum_hi, carry_hi1) = ah.overflowing_add(bh);
            let mut cy = u8::from(carry_hi1);
            if carry_lo {
                let (s, c) = sum_hi.overflowing_add(1);
                sum_hi = s;
                cy += u8::from(c);
            }
            let _ = &mut sum_lo;
            if cy != 0 {
                // Carry out of bit 255: shift right one, restore bit 255.
                let new_lo = (sum_hi << 127) | (sum_lo >> 1);
                let new_hi = (1u128 << 127) | (sum_hi >> 1);
                ch = new_hi;
                cl = new_lo;
                r_ex += 1;
            } else {
                ch = sum_hi;
                cl = sum_lo;
            }
        }

        Self {
            sgn,
            ex: r_ex,
            hi: ch,
            lo: cl,
        }
    }

    /// Round-to-nearest sum considering only the upper two limbs (port of
    /// `add_qint_22`); result's lower half is zero.  Error < 2 ulps(128).
    #[inline]
    pub fn add_22(&self, other: &Self) -> Self {
        if self.is_zero() {
            return *other;
        }
        if other.is_zero() {
            return *self;
        }

        let (a, b) = match self.cmp_22(other) {
            core::cmp::Ordering::Equal => {
                if self.sgn != other.sgn {
                    return ZERO_Q;
                }
                return Self {
                    ex: self.ex + 1,
                    ..*self
                };
            }
            core::cmp::Ordering::Less => (other, self),
            core::cmp::Ordering::Greater => (self, other),
        };

        let mut ah = a.hi;
        let m_ex = a.ex;
        let k = (a.ex - b.ex) as u32;
        let mut bh = if k >= 128 { 0 } else { b.hi >> k };

        let sgn = a.sgn;
        let mut r_ex = m_ex;
        let ch;

        if a.sgn != b.sgn {
            let mut c = ah.wrapping_sub(bh);
            let mut ex = c.leading_zeros();
            if ex > 0 {
                ah <<= ex;
                bh = if ex >= k {
                    b.hi << (ex - k)
                } else {
                    b.hi >> (k - ex)
                };
                r_ex -= i64::from(ex);
                c = ah.wrapping_sub(bh);
                ex = c.leading_zeros();
            }
            ch = c << ex;
            r_ex -= i64::from(ex);
        } else {
            let (sum, carry) = ah.overflowing_add(bh);
            if carry {
                ch = (1u128 << 127) | (sum >> 1);
                r_ex += 1;
            } else {
                ch = sum;
            }
        }

        Self {
            sgn,
            ex: r_ex,
            hi: ch,
            lo: 0,
        }
    }

    /// Full 256×256→256 product, error < 14 ulps (port of `mul_qint`).
    #[inline]
    pub fn mul(&self, b: &Self) -> Self {
        let (ahh, ahl, alh, all) = limbs(self.hi, self.lo);
        let (bhh, bhl, blh, bll) = limbs(b.hi, b.lo);
        let p = |x: u64, y: u64| u128::from(x) * u128::from(y);

        let r33 = p(ahh, bhh);
        let r32 = p(ahh, bhl);
        let r23 = p(ahl, bhh);
        let r31 = p(ahh, blh);
        let r13 = p(alh, bhh);
        let r22 = p(ahl, bhl);
        let r30 = p(ahh, bll);
        let r03 = p(all, bhh);
        let r21 = p(ahl, blh);
        let r12 = p(alh, bhl);

        let t3 = (r12 >> 64) + (r21 >> 64) + (r03 >> 64) + (r30 >> 64);

        let (mut t4, mut c4) = add_carry(r22, t3);
        let (nt4, cc) = add_carry(r13, t4);
        t4 = nt4;
        c4 += cc;
        let (nt4b, cc2) = add_carry(r31, t4);
        t4 = nt4b;
        c4 += cc2;

        let (mut t5, mut c5) = add_carry(r23, t4 >> 64);
        let (nt5, cc3) = add_carry(r32, t5);
        t5 = nt5;
        c5 += cc3;

        let t6 = r33.wrapping_add((c5 << 64) | (t5 >> 64)).wrapping_add(c4);

        finish_mul(self.sgn ^ b.sgn, self.ex + b.ex, t6, t5, t4)
    }

    /// Product using only the upper 3 limbs of each operand, error < 6 ulps
    /// (port of `mul_qint_33`).
    #[inline]
    pub fn mul_33(&self, b: &Self) -> Self {
        let (ahh, ahl, alh, _) = limbs(self.hi, self.lo);
        let (bhh, bhl, blh, _) = limbs(b.hi, b.lo);
        let p = |x: u64, y: u64| u128::from(x) * u128::from(y);

        let r33 = p(ahh, bhh);
        let r32 = p(ahh, bhl);
        let r23 = p(ahl, bhh);
        let r31 = p(ahh, blh);
        let r13 = p(alh, bhh);
        let r22 = p(ahl, bhl);
        let r21 = p(ahl, blh);
        let r12 = p(alh, bhl);

        let t3 = (r12 >> 64) + (r21 >> 64);

        let (mut t4, mut c4) = add_carry(r22, t3);
        let (nt4, cc) = add_carry(r13, t4);
        t4 = nt4;
        c4 += cc;
        let (nt4b, cc2) = add_carry(r31, t4);
        t4 = nt4b;
        c4 += cc2;

        let (mut t5, mut c5) = add_carry(r23, t4 >> 64);
        let (nt5, cc3) = add_carry(r32, t5);
        t5 = nt5;
        c5 += cc3;

        let t6 = r33.wrapping_add((c5 << 64) | (t5 >> 64)).wrapping_add(c4);

        finish_mul(self.sgn ^ b.sgn, self.ex + b.ex, t6, t5, t4)
    }

    /// Product using only the upper limb of `b`, error < 2 ulps
    /// (port of `mul_qint_41`).
    #[inline]
    pub fn mul_41(&self, b: &Self) -> Self {
        let (ahh, ahl, alh, all) = limbs(self.hi, self.lo);
        let bhh = (b.hi >> 64) as u64;
        let p = |x: u64, y: u64| u128::from(x) * u128::from(y);

        let r33 = p(ahh, bhh);
        let r23 = p(ahl, bhh);
        let r13 = p(alh, bhh);
        let r03 = p(all, bhh);

        let t3 = r03 >> 64;
        let (t4, c4) = add_carry(r13, t3);
        let (t5, c5) = add_carry(r23, t4 >> 64);
        let t6 = r33.wrapping_add((c5 << 64) | (t5 >> 64)).wrapping_add(c4);

        finish_mul(self.sgn ^ b.sgn, self.ex + b.ex, t6, t5, t4)
    }

    /// Exact product, upper 3 limbs of `self` × upper limb of `b`
    /// (port of `mul_qint_31`).
    #[inline]
    pub fn mul_31(&self, b: &Self) -> Self {
        let (ahh, ahl, alh, _) = limbs(self.hi, self.lo);
        let bhh = (b.hi >> 64) as u64;
        let p = |x: u64, y: u64| u128::from(x) * u128::from(y);

        let r33 = p(ahh, bhh);
        let r23 = p(ahl, bhh);
        let r13 = p(alh, bhh);

        let t4 = r13;
        let (t5, c5) = add_carry(r23, t4 >> 64);
        let t6 = r33.wrapping_add((c5 << 64) | (t5 >> 64));

        finish_mul(self.sgn ^ b.sgn, self.ex + b.ex, t6, t5, t4)
    }

    /// Exact product using the upper two limbs of each operand
    /// (port of `mul_qint_22`).
    #[inline]
    pub fn mul_22(&self, b: &Self) -> Self {
        let (ahh, ahl, _, _) = limbs(self.hi, self.lo);
        let (bhh, bhl, _, _) = limbs(b.hi, b.lo);
        let p = |x: u64, y: u64| u128::from(x) * u128::from(y);

        let r33 = p(ahh, bhh);
        let r32 = p(ahh, bhl);
        let r23 = p(ahl, bhh);
        let r22 = p(ahl, bhl);

        let t4 = r22;
        let (mut t5, mut c5) = add_carry(r23, t4 >> 64);
        let (nt5, cc) = add_carry(r32, t5);
        t5 = nt5;
        c5 += cc;
        let t6 = r33.wrapping_add((c5 << 64) | (t5 >> 64));

        finish_mul(self.sgn ^ b.sgn, self.ex + b.ex, t6, t5, t4)
    }

    /// Exact product, upper two limbs of `self` × upper limb of `b`
    /// (port of `mul_qint_21`).
    #[inline]
    pub fn mul_21(&self, b: &Self) -> Self {
        let (ahh, ahl, _, _) = limbs(self.hi, self.lo);
        let bhh = (b.hi >> 64) as u64;
        let p = |x: u64, y: u64| u128::from(x) * u128::from(y);

        let r33 = p(ahh, bhh);
        let r23 = p(ahl, bhh);

        let t6 = r33 + (r23 >> 64);
        let t5 = r23 << 64;
        finish_mul(self.sgn ^ b.sgn, self.ex + b.ex, t6, t5, 0)
    }

    /// Exact product using only the upper limb of each operand
    /// (port of `mul_qint_11`).
    #[inline]
    pub fn mul_11(&self, b: &Self) -> Self {
        let ahh = (self.hi >> 64) as u64;
        let bhh = (b.hi >> 64) as u64;
        let t6 = u128::from(ahh) * u128::from(bhh);
        let ex = u32::from((t6 >> 127) == 0);
        Self {
            sgn: self.sgn ^ b.sgn,
            ex: self.ex + b.ex + 1 - i64::from(ex),
            hi: t6 << ex,
            lo: 0,
        }
    }

    /// Product with a signed integer, error < 2 ulps (port of `mul_qint_2`).
    #[inline]
    pub fn mul_int(&self, b: i64) -> Self {
        if b == 0 {
            return ZERO_Q;
        }
        let mut c = b.unsigned_abs();
        let sgn = (b < 0) ^ self.sgn;
        if c == 1 {
            return Self { sgn, ..*self };
        }

        let mut r_ex = self.ex + 64;
        let k = c.leading_zeros();
        c <<= k;
        r_ex -= i64::from(k);

        let (ahh, ahl, alh, all) = limbs(self.hi, self.lo);
        let cc = u128::from(c);
        let t3 = u128::from(ahh) * cc;
        let t2 = u128::from(ahl) * cc;
        let t1 = u128::from(alh) * cc;
        let t0 = u128::from(all) * cc;

        let t = t0 >> 64;
        let (mut t1v, cy1) = add_carry(t, t1);
        let tmid = ((cy1) << 64) | (t1v >> 64);
        let (mut t2v, cy2) = add_carry(tmid, t2);
        let t3v = t3.wrapping_add(((cy2) << 64) | (t2v >> 64));

        let ex = ((t3v >> 64) as u64).leading_zeros();
        t2v = (t2v << 64) | (t1v & u128::from(u64::MAX));
        let _ = &mut t1v;

        let (hi, lo, dex);
        if ex != 0 {
            hi = (t3v << 1) | (t2v >> 127);
            lo = t2v << 1;
            dex = -1;
        } else {
            hi = t3v;
            lo = t2v;
            dex = 0;
        }

        Self {
            sgn,
            ex: r_ex + dex,
            hi,
            lo,
        }
    }

    /// Convert a finite nonzero `f64` to a `Qint` (port of `qint_fromd`).
    #[inline]
    pub fn from_f64(b: f64) -> Self {
        let bits = b.to_bits();
        let biased = ((bits >> 52) & 0x7ff) as i64;
        // Significand with the implicit leading bit restored (assumes b normal,
        // which the pow accurate path guarantees: x, y are finite nonzero, and
        // x is already normalized to [MIN_POSITIVE, ∞) before this call).
        let hh = (bits & (!0u64 >> 12)) + if biased != 0 { 1u64 << 52 } else { 0 };
        let ex = biased - 0x3ff;
        let t = hh.leading_zeros();
        Self {
            sgn: b < 0.0,
            ex: ex - i64::from(if t > 11 { t - 12 } else { 0 }),
            hi: u128::from(hh << t) << 64,
            lo: 0,
        }
    }

    /// Truncate toward zero to an `i64` (port of `qint_toi`).
    #[inline]
    pub fn to_i64(&self) -> i64 {
        if self.ex < 0 {
            return 0;
        }
        let hh = (self.hi >> 64) as u64;
        let r = (hh >> (63 - self.ex)) as i64;
        if self.sgn { -r } else { r }
    }

    /// Round to `f64` (round-to-nearest, port of `qint_tod` + `subnormalize_qint`).
    ///
    /// Handles the subnormal range and overflow saturation; metallic is RNDN
    /// only, so the directed-rounding cases of the C are dropped.
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    pub fn to_f64(&self) -> f64 {
        let mut a = *self;
        // subnormalize_qint: round the significand onto the subnormal grid.
        if a.ex <= -1023 {
            let ex = (-(1011 + a.ex)) as u32;
            let hh = (a.hi >> 64) as u64;
            let hl = a.hi as u64;
            let (lh, ll) = ((a.lo >> 64) as u64, a.lo as u64);
            // `ex` can exceed 63 here; mirror the C's well-defined-shift regime.
            let hi_shift = if ex >= 64 { 0 } else { hh >> ex };
            let md = if ex == 0 {
                0
            } else if ex > 64 {
                0
            } else {
                (hh >> (ex - 1)) & 1
            };
            let lo_sticky = (ex < 64 && (hh & (u64::MAX >> ex)) != 0)
                || (ex >= 64 && hh != 0)
                || hl != 0
                || lh != 0
                || ll != 0;
            let mut hi = hi_shift;
            // round to nearest, ties to even
            hi += if lo_sticky { md } else { hi & md };
            let mut new_hh = hi << ex.min(127);
            let mut new_ex = a.ex;
            if new_hh == 0 {
                new_ex += 1;
                new_hh = 1u64 << 63;
            }
            a.hi = u128::from(new_hh) << 64;
            a.lo = 0;
            a.ex = new_ex;
        }

        let hh = (a.hi >> 64) as u64;
        let mut r = f64::from_bits((hh >> 11) | (0x3ffu64 << 52));

        let mut rd = 0.0_f64;
        if hh & 0x400 != 0 {
            rd += f64::from_bits(0x3ca0_0000_0000_0000); // 2^-53
        }
        if hh & 0x3ff != 0 || (a.hi as u64) != 0 || a.lo != 0 {
            rd += f64::from_bits(0x3c90_0000_0000_0000); // 2^-54
        }
        if a.sgn {
            r = f64::from_bits(r.to_bits() | (1u64 << 63));
            rd = -rd;
        }
        r += rd;

        let e: f64;
        if a.ex > -1023 {
            if a.ex > 1023 {
                if a.ex == 1024 {
                    r *= 2.0;
                    e = f64::from_bits(0x7fe0_0000_0000_0000); // 2^1023
                } else {
                    r = f64::from_bits(0x7fef_ffff_ffff_ffff); // 0x1.f...p1023
                    e = f64::from_bits(0x7fef_ffff_ffff_ffff);
                }
            } else {
                e = f64::from_bits((((a.ex + 1023) & 0x7ff) as u64) << 52);
            }
        } else if a.ex < -1074 {
            if a.ex == -1075 {
                r *= 0.5;
                e = f64::from_bits(1); // 2^-1074
            } else {
                r = f64::from_bits(1); // 0x0.0..1p-1022
                e = f64::from_bits(1);
            }
        } else {
            e = f64::from_bits(1u64 << ((a.ex + 1074) as u32));
        }

        r * e
    }
}

/// Count leading zeros across a 256-bit value `(hi:128, lo:128)`.
#[inline]
fn clz256(hi: u128, lo: u128) -> u32 {
    if hi != 0 {
        hi.leading_zeros()
    } else {
        128 + lo.leading_zeros()
    }
}

/// Shift a 256-bit value `(hi, lo)` left by `ex` (`0 ≤ ex < 256`).
#[inline]
fn normalize_left(hi: u128, lo: u128, ex: u32) -> (u128, u128) {
    if ex == 0 {
        (hi, lo)
    } else if ex >= 128 {
        (lo << (ex - 128), 0)
    } else {
        ((hi << ex) | (lo >> (128 - ex)), lo << ex)
    }
}

/// `a + b` returning `(sum, carry)` with the carry as a `u128` (0 or 1).
#[inline]
fn add_carry(a: u128, b: u128) -> (u128, u128) {
    let (s, c) = a.overflowing_add(b);
    (s, u128::from(c))
}

/// Common tail of the `mul_qint*` family: normalize `(t6, t5, t4)` so bit 255 is
/// set, assemble the 256-bit significand, set sign/exponent.
///
/// `t6` is the top 128 bits, `t5` the next 128, and `t4`'s low 64 bits extend
/// `t5` (`mul_qint`'s `t5 = (t5 << 64) | low(t4)`).
#[inline]
fn finish_mul(sgn: bool, ex_sum: i64, t6: u128, t5: u128, t4: u128) -> Qint {
    let ex = u32::from((t6 >> 127) == 0);
    let t5 = (t5 << 64) | (t4 & u128::from(u64::MAX));
    let (hi, lo) = if ex == 1 {
        ((t6 << 1) | (t5 >> 127), t5 << 1)
    } else {
        (t6, t5)
    };
    Qint {
        sgn,
        ex: ex_sum + 1 - i64::from(ex),
        hi,
        lo,
    }
}

#[cfg(test)]
mod tests {
    use super::super::pow_consts::{LOG2_Q, ONE_Q};
    use super::*;

    /// Reference 256-bit value of a `Qint` as a `rug::Float` (mpfr feature only).
    #[cfg(feature = "mpfr")]
    fn to_big(q: &Qint) -> rug::Float {
        use rug::Float;
        use rug::ops::Pow;
        let m = Float::with_val(300, q.hi)
            * Float::with_val(300, &Float::with_val(300, 2).pow(128))
            + Float::with_val(300, q.lo);
        let val = m / Float::with_val(300, &Float::with_val(300, 2).pow(255))
            * Float::with_val(300, &Float::with_val(300, 2).pow(q.ex));
        if q.sgn { -val } else { val }
    }

    #[test]
    fn one_q_is_one() {
        assert_eq!(ONE_Q.to_f64(), 1.0);
    }

    #[test]
    fn from_f64_round_trip_small() {
        for &v in &[1.0_f64, 2.0, 0.5, 3.0, 1.5, 1234.5, 1e300, 1e-300] {
            let q = Qint::from_f64(v);
            assert_eq!(q.to_f64(), v, "round trip failed for {v}");
        }
    }

    #[test]
    fn mul_11_basic() {
        // 2 * 3 = 6 via mul_11 (single-limb operands).
        let two = Qint::from_f64(2.0);
        let three = Qint::from_f64(3.0);
        assert_eq!(two.mul_11(&three).to_f64(), 6.0);
        assert_eq!(two.mul(&three).to_f64(), 6.0);
    }

    #[test]
    fn add_basic() {
        let a = Qint::from_f64(1.5);
        let b = Qint::from_f64(0.25);
        assert_eq!(a.add(&b).to_f64(), 1.75);
        let c = Qint::from_f64(-0.5);
        assert_eq!(a.add(&c).to_f64(), 1.0);
    }

    /// LOG2_Q must approximate ln 2 to full f64 precision.
    #[test]
    fn log2_q_value() {
        assert_eq!(LOG2_Q.to_f64(), core::f64::consts::LN_2);
    }

    /// Cross-check the qint primitives against `rug` at 256-bit precision.
    #[cfg(feature = "mpfr")]
    #[test]
    fn qint_ops_vs_rug() {
        use rug::Float;
        let mut state = 0x1234_5678_9abc_def0u64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let mut worst_mul = 0.0_f64;
        let mut worst_add = 0.0_f64;
        for _ in 0..200_000 {
            let xb = (next() & 0x7fff_ffff_ffff_ffff) | (1u64 << 62);
            let yb = (next() & 0x7fff_ffff_ffff_ffff) | (1u64 << 62);
            let x = f64::from_bits(xb).abs() + 1e-300;
            let y = f64::from_bits(yb).abs() + 1e-300;
            if !(x.is_finite() && y.is_finite() && x > 0.0 && y > 0.0) {
                continue;
            }
            let qx = Qint::from_f64(x);
            let qy = Qint::from_f64(y);

            // Multiply.
            let prod = qx.mul(&qy);
            let truth = Float::with_val(300, x) * Float::with_val(300, y);
            let got = to_big(&prod);
            let rel = (Float::with_val(300, &got - &truth) / &truth)
                .abs()
                .to_f64();
            worst_mul = worst_mul.max(rel);

            // Add.
            let sum = qx.add(&qy);
            let truth_a = Float::with_val(300, x) + Float::with_val(300, y);
            let got_a = to_big(&sum);
            let rel_a = (Float::with_val(300, &got_a - &truth_a) / &truth_a)
                .abs()
                .to_f64();
            worst_add = worst_add.max(rel_a);
        }
        // 14 ulps(256) ≈ 2^-251.2; allow generous slack.
        assert!(worst_mul < 2.0_f64.powi(-245), "mul rel err {worst_mul:e}");
        assert!(worst_add < 2.0_f64.powi(-250), "add rel err {worst_add:e}");
    }
}
