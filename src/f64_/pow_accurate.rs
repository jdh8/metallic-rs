//! Correctly-rounded accurate path for f64 `pow`, ported from CORE-MATH's
//! `binary64/pow/pow.c` (reference [5]: Hubrecht, Jeannerod, Zimmermann,
//! ARITH 2023).
//!
//! [`pow_accurate`] reproduces CORE-MATH's two final Ziv iterations and its
//! exact/midpoint detector:
//!
//! * **Phase 2** — a 128-bit [`Dint`] chain `log_2 → ×y → exp_2`, relative error
//!   ≈2⁻¹¹³, with a rounding test (`ERR_BND_2 = 28`).
//! * **Exact detection** — [`exact_pow`] recognizes the rational-exponent powers
//!   whose `xʸ` is exactly representable or an exact midpoint (`y` integer in
//!   `2..=34`, or `x = 2^E·m`, `y = 2^F·n` with `m, n` odd) and returns the
//!   exact `f64`, resolving the ties-to-even cases no finite approximation can.
//! * **Phase 3** — a 256-bit [`Qint`] chain `log_3 → ×y → exp_3`, relative error
//!   ≈2⁻²⁴⁰, with a rounding test (`ERR_BND_3 = 60`) and the near-1 tail.
//!
//! metallic is round-to-nearest only, so the directed-rounding and errno/fenv
//! side effects of the C are dropped; the final `Dint`/`Qint` → `f64` rounders
//! ([`super::dint::Dint::to_f64_general`]-style and [`Qint::to_f64`]) match the
//! C's RNDN result bit-for-bit, including the subnormal range and overflow.
//!
//! The dint phase here uses pow.c's own `add_dint`/`mul_dint_*` semantics
//! (faithful free functions below), which differ from the log family's
//! [`Dint`](super::dint::Dint) methods (a different reduction); only the struct
//! and the 6-ulp [`Dint::mul`](super::dint::Dint::mul) (= pow.c `mul_dint`) are
//! shared.
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap, clippy::similar_names)]
#![allow(clippy::many_single_char_names, clippy::too_many_lines)]
// This module mirrors CORE-MATH's C line-for-line; keep the C's branch and range
// structure (which trips these style lints) so the port stays auditable.
#![allow(clippy::manual_range_contains, clippy::if_same_then_else)]
#![allow(clippy::missing_const_for_fn, clippy::branches_sharing_code)]

use super::dint::Dint;
use super::pow_consts as c;
use super::qint::Qint;

// ===========================================================================
// dint64_t helper operations (pow.c's `dint.h`, faithful ports).
//
// These take the canonical normalized `Dint { sgn, ex, m }` with `m = hi<<64|lo`
// and reproduce CORE-MATH's rounding exactly.  They are distinct from the log
// family's `Dint` methods in `dint.rs` (which implement a different reduction);
// `Dint::mul` is shared because it already equals pow.c's `mul_dint`.
// ===========================================================================

/// `true` iff zero (top word is zero ⇒ normalized zero).
#[inline]
const fn d_is_zero(a: &Dint) -> bool {
    (a.m >> 64) == 0
}

/// Compare magnitudes (port of `cmp_dint_abs`).
#[inline]
fn d_cmp_abs(a: &Dint, b: &Dint) -> core::cmp::Ordering {
    if d_is_zero(a) {
        return if d_is_zero(b) {
            core::cmp::Ordering::Equal
        } else {
            core::cmp::Ordering::Less
        };
    }
    if d_is_zero(b) {
        return core::cmp::Ordering::Greater;
    }
    a.ex.cmp(&b.ex).then_with(|| a.m.cmp(&b.m))
}

/// Round-to-nearest sum (port of `add_dint`), exact when Sterbenz applies.
#[inline]
fn d_add(a: &Dint, b: &Dint) -> Dint {
    if d_is_zero(a) {
        return *b;
    }
    if d_is_zero(b) {
        return *a;
    }

    let (a, b) = match d_cmp_abs(a, b) {
        core::cmp::Ordering::Equal => {
            if a.sgn != b.sgn {
                return c::ZERO_D;
            }
            return Dint { ex: a.ex + 1, ..*a };
        }
        core::cmp::Ordering::Less => (b, a),
        core::cmp::Ordering::Greater => (a, b),
    };

    // |A| > |B| ⇒ a.ex >= b.ex.
    let big_a = a.m;
    let mut bb = b.m;
    let k = (a.ex - b.ex) as u32;
    if k > 0 {
        bb = if k < 128 { bb >> k } else { 0 };
    }

    let sgn = a.sgn;
    let mut r_ex = a.ex;
    let c_out: u128;

    if a.sgn != b.sgn {
        // Subtraction C = A - B.
        let mut cc = big_a.wrapping_sub(bb);
        // C != 0 since |A| > |B|.
        let mut ex = cc.leading_zeros();
        if ex > 0 {
            if k == 1 {
                // Sterbenz: re-subtract from the full b (lo bits intact).
                cc = (big_a << ex).wrapping_sub(b.m << (ex - 1));
            } else {
                cc = (big_a << ex).wrapping_sub(bb << ex);
            }
            r_ex -= i64::from(ex);
            ex = cc.leading_zeros();
        }
        cc <<= ex;
        r_ex -= i64::from(ex);
        c_out = cc;
    } else {
        // Addition.
        let (sum, carry) = big_a.overflowing_add(bb);
        if carry {
            c_out = (1u128 << 127) | (sum >> 1);
            r_ex += 1;
        } else {
            c_out = sum;
        }
    }

    Dint {
        sgn,
        ex: r_ex,
        m: c_out,
    }
}

/// Round-to-nearest sum assuming both low words zero (port of `add_dint_11`);
/// operates on the high 64-bit words only, result's low word is zero.
#[inline]
fn d_add_11(a: &Dint, b: &Dint) -> Dint {
    let ahi = (a.m >> 64) as u64;
    let bhi0 = (b.m >> 64) as u64;
    if ahi == 0 {
        return *b;
    }
    if bhi0 == 0 {
        return *a;
    }

    // Compare by (ex, hi) only (cmp_dint_11).
    let (a, b, ahi, bhi0) = match a.ex.cmp(&b.ex).then_with(|| ahi.cmp(&bhi0)) {
        core::cmp::Ordering::Equal => {
            if a.sgn != b.sgn {
                return c::ZERO_D;
            }
            return Dint { ex: a.ex + 1, ..*a };
        }
        core::cmp::Ordering::Less => (b, a, bhi0, ahi),
        core::cmp::Ordering::Greater => (a, b, ahi, bhi0),
    };

    // |A| > |B| ⇒ a.ex >= b.ex.
    let big_a = ahi;
    let mut bb = bhi0;
    if a.ex > b.ex {
        let k = (a.ex - b.ex) as u32;
        bb = if k < 64 { bb >> k } else { 0 };
    }

    let sgn = a.sgn;
    let mut r_ex = a.ex;
    let cc: u64;

    if a.sgn != b.sgn {
        let mut c0 = big_a.wrapping_sub(bb);
        // C != 0 since |A| > |B|.
        let mut ex = c0.leading_zeros();
        if ex > 0 {
            c0 = (big_a << ex).wrapping_sub(bb << ex);
            r_ex -= i64::from(ex);
            ex = c0.leading_zeros();
        }
        cc = c0 << ex;
        r_ex -= i64::from(ex);
    } else {
        let (sum, carry) = big_a.overflowing_add(bb);
        if carry {
            cc = (1u64 << 63) | (sum >> 1);
            r_ex += 1;
        } else {
            cc = sum;
        }
    }

    Dint {
        sgn,
        ex: r_ex,
        m: u128::from(cc) << 64,
    }
}

/// Exact product assuming both low words zero (port of `mul_dint_11`).
#[inline]
fn d_mul_11(a: &Dint, b: &Dint) -> Dint {
    let ahi = (a.m >> 64) as u64;
    let bhi = (b.m >> 64) as u64;
    let mut prod = u128::from(ahi) * u128::from(bhi);
    // `shift` = 1 when the top bit is clear (needs a 1-bit left normalize).
    let shift = u32::from((prod >> 127) == 0);
    prod <<= shift;
    Dint {
        sgn: a.sgn ^ b.sgn,
        ex: a.ex + b.ex + 1 - i64::from(shift),
        m: prod,
    }
}

/// Product assuming `b`'s low word is zero (port of `mul_dint_21`), error ≤ 2 ulps.
#[inline]
fn d_mul_21(a: &Dint, b: &Dint) -> Dint {
    let ahi = u128::from((a.m >> 64) as u64);
    let alo = u128::from(a.m as u64);
    let bhi = u128::from((b.m >> 64) as u64);
    let hi = ahi * bhi;
    let lo = alo * bhi;
    let mut r = hi + (lo >> 64);
    let ex = (r >> 127) as u64;
    r <<= 1 - ex;
    Dint {
        sgn: a.sgn ^ b.sgn,
        ex: a.ex + b.ex + ex as i64,
        m: r,
    }
}

/// Product with a signed integer, error < 1 ulp (port of `mul_dint_int64`).
#[inline]
fn d_mul_int(a: &Dint, b: i64) -> Dint {
    if b == 0 {
        return c::ZERO_D;
    }
    let cmag = b.unsigned_abs();
    let sgn = if b < 0 { !a.sgn } else { a.sgn };
    let mut r_ex = a.ex + 64;

    let ahi = (a.m >> 64) as u64;
    let alo = a.m as u64;
    let mut r = u128::from(ahi) * u128::from(cmag);

    // c=1 may leave the high word zero.
    let m = if (r >> 64) != 0 {
        ((r >> 64) as u64).leading_zeros()
    } else {
        64
    };
    r <<= m;
    r_ex -= i64::from(m);

    let l = (u128::from(alo) * u128::from(cmag)) << (m - 1) >> 63;
    let (sum, carry) = r.overflowing_add(l);
    r = sum;
    if carry {
        r = (1u128 << 127) | (r >> 1);
        r_ex += 1;
    }

    Dint {
        sgn,
        ex: r_ex,
        m: r,
    }
}

/// Truncate toward zero to `i64` (port of `dint_toi`).
#[inline]
fn d_toi(a: &Dint) -> i64 {
    if a.ex < 0 {
        return 0;
    }
    let hi = (a.m >> 64) as u64;
    let r = (hi >> (63 - a.ex)) as i64;
    if a.sgn { -r } else { r }
}

/// Round-to-nearest `f64`, full range incl. subnormal/overflow (port of
/// `dint_tod` + `dint_tod_subnormal`, RNDN branch only).
#[inline]
fn d_tod(a: &Dint) -> f64 {
    let hi = (a.m >> 64) as u64;
    let lo = a.m as u64;

    if a.ex < -1022 {
        // Subnormal range (port of `dint_tod_subnormal`, RNDN).  `ex >= 12`; the
        // `ex >= 64` arm below covers everything below `2^-1074` including
        // `a.ex < -1075` (the whole significand is then sticky → 0 or 2^-1074).
        let ex = (-(1011 + a.ex)) as u64;
        let mut ret: f64;
        if ex >= 64 {
            // |a| < 2^-1074.
            let rb = hi >> 63;
            let sb = (hi << 1) | lo;
            ret = if ex > 64 || rb == 0 || sb == 0 {
                0.0
            } else {
                f64::from_bits(1) // 2^-1074
            };
            if a.sgn {
                ret = -ret;
            }
            return ret;
        }
        let exu = ex as u32;
        let mut h = hi >> exu;
        let rb = (hi >> (exu - 1)) & 1;
        let sb = u64::from((hi << (65 - exu)) != 0) | u64::from(lo != 0);
        h += if sb != 0 { rb } else { h & rb };
        ret = f64::from_bits(h | (u64::from(a.sgn) << 63));
        return ret;
    }

    // r = significand in [1, 2).
    let mut r = f64::from_bits((hi >> 11) | (0x3ffu64 << 52) | (u64::from(a.sgn) << 63));
    let mut rd = 0.0_f64;
    if (hi >> 10) & 1 == 1 {
        rd += f64::from_bits(0x3ca0_0000_0000_0000); // 2^-53
    }
    if hi & 0x3ff != 0 || lo != 0 {
        rd += f64::from_bits(0x3c90_0000_0000_0000); // 2^-54
    }
    if a.sgn {
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
                r = f64::from_bits(0x7fef_ffff_ffff_ffff);
                e = f64::from_bits(0x7fef_ffff_ffff_ffff);
            }
        } else {
            e = f64::from_bits((((a.ex + 1023) & 0x7ff) as u64) << 52);
        }
    } else if a.ex < -1074 {
        if a.ex == -1075 {
            r *= 0.5;
            e = f64::from_bits(1);
        } else {
            r = f64::from_bits(1);
            e = f64::from_bits(1);
        }
    } else {
        e = f64::from_bits(1u64 << ((a.ex + 1074) as u32));
    }
    r * e
}

// ===========================================================================
// Phase 2: 128-bit dint accurate path.
// ===========================================================================

/// `exp(z)` for `|z| < 0.00016923` as a `Dint` (port of `q_2`), rel err < 2⁻¹²²·²⁹.
#[inline]
fn q_2(y: &Dint) -> Dint {
    let mut r = d_mul_11(y, &c::Q_2[0]);
    r = d_add_11(&c::Q_2[1], &r);
    r = d_mul_11(y, &r);
    r = d_add_11(&c::Q_2[2], &r);
    r = d_mul_11(y, &r);
    r = d_add(&c::Q_2[3], &r);
    r = y.mul(&r);
    r = d_add(&c::Q_2[4], &r);
    r = y.mul(&r);
    r = d_add(&c::Q_2[5], &r);
    r = y.mul(&r);
    r = d_add(&c::Q_2[6], &r);
    r = y.mul(&r);
    d_add(&c::Q_2[7], &r)
}

/// `log(1+z)` for `|z| <= 2^-13`, low word of `z` zero (port of `p_2`).
#[inline]
fn p_2(z: &Dint) -> Dint {
    let mut r = d_mul_11(z, &c::P_2[0]);
    r = d_add_11(&c::P_2[1], &r);
    r = d_mul_11(z, &r);
    r = d_add_11(&c::P_2[2], &r);
    r = d_mul_11(z, &r);
    r = d_add_11(&c::P_2[3], &r);
    r = d_mul_11(z, &r);
    r = d_add(&c::P_2[4], &r);
    r = d_mul_21(&r, z);
    r = d_add(&c::P_2[5], &r);
    r = d_mul_21(&r, z);
    r = d_add(&c::P_2[6], &r);
    r = d_mul_21(&r, z);
    r = d_add(&c::P_2[7], &r);
    r = d_mul_21(&r, z);
    r = d_add(&c::P_2[8], &r);
    d_mul_21(&r, z)
}

/// `log(x)` as a `Dint`, relative error < 2⁻¹²²·⁸⁸ (port of `log_2`).
#[inline]
fn log_2(x: &Dint) -> Dint {
    let mut x = *x;
    let mut big_e = x.ex;
    let hi = (x.m >> 64) as u64;
    let i: usize = if hi > 0xb504_f333_f9de_6484 {
        big_e += 1;
        (hi >> (63 + 1 - 7)) as usize
    } else {
        (hi >> (63 - 7)) as usize
    };
    // now 90 <= i <= 181
    x.ex -= big_e;

    let mut z = d_mul_11(&x, &c::INVERSE_2_1[i - 90]); // exact
    // 2nd index j = floor(z * 2^13); z.ex in {0,-1}.
    let zhi = (z.m >> 64) as u64;
    let j = (zhi >> ((63 - 13 - z.ex) as u32)) as usize;
    z = d_mul_11(&z, &c::INVERSE_2_2[j - 8128]); // exact
    z = d_add(&c::M_ONE_D, &z); // exact (subtract 1)

    let r = d_mul_int(&c::LOG2_D, big_e);
    let mut p = p_2(&z);
    p = d_add(&c::LOG_INV_2_2[j - 8128], &p);
    p = d_add(&c::LOG_INV_2_1[i - 90], &p);
    d_add(&p, &r)
}

/// `exp(x)` as a `Dint`, for `|x| < 744.45`, rel err < 2⁻¹²¹·⁷⁰ (port of `exp_2`).
#[inline]
fn exp_2(x: &Dint) -> Dint {
    if x.ex >= 10 {
        // Underflow or overflow.
        let mut r = *x;
        r.ex = if x.sgn { -1076 } else { 1025 };
        r.sgn = false;
        return r;
    }

    let big_k = d_mul_11(x, &c::LOG2_INV_D); // exact (low word of x assumed 0)
    let k = d_toi(&big_k); // trunc toward zero
    let mut kk = d_mul_int(&c::LOG2_D, k);
    kk.ex -= 12;
    kk.sgn = !kk.sgn;
    let y = d_add(x, &kk); // exact (Sterbenz)

    let big_m = k >> 12;
    let i2 = ((k >> 6) & 0x3f) as usize;
    let i1 = (k & 0x3f) as usize;

    let mut r = q_2(&y);
    r = c::T1_2[i2].mul(&r);
    r = c::T2_2[i1].mul(&r);
    r.ex += big_m;
    r
}

// ===========================================================================
// Phase 3: 256-bit qint accurate path.
// ===========================================================================

/// `exp(z)` for `|z| < 0.00016923` as a `Qint` (port of `q_3`).
#[inline]
fn q_3(y: &Qint) -> Qint {
    let mut r = y.mul_11(&c::Q_3[0]);
    r = c::Q_3[1].add_22(&r);
    let mut k = 2;
    while k < 7 {
        r = y.mul_22(&r);
        r = c::Q_3[k].add_22(&r);
        k += 1;
    }
    while k < 12 {
        r = y.mul_33(&r);
        r = c::Q_3[k].add(&r);
        k += 1;
    }
    while k < 15 {
        r = y.mul(&r);
        r = c::Q_3[k].add(&r);
        k += 1;
    }
    r
}

/// `log(1+z)` for `|z| <= 2^-13`, upper limb only (port of `p_3`).
#[inline]
fn p_3(z: &Qint) -> Qint {
    let mut r = c::P_3[0].mul_11(z);
    r = c::P_3[1].add_22(&r);
    for k in 2..4 {
        r = r.mul_11(z);
        r = c::P_3[k].add_22(&r);
    }
    for k in 4..8 {
        r = r.mul_21(z);
        r = c::P_3[k].add_22(&r);
    }
    for k in 8..14 {
        r = r.mul_31(z);
        r = c::P_3[k].add(&r);
    }
    for k in 14..18 {
        r = r.mul_41(z);
        r = c::P_3[k].add(&r);
    }
    r.mul_41(z)
}

/// `log(x)` as a `Qint`, relative error < 2⁻²⁵⁰·⁷⁴ (port of `log_3`).
#[inline]
fn log_3(x: &Qint) -> Qint {
    let mut x = *x;
    let mut big_e = x.ex;
    let hh = (x.hi >> 64) as u64;
    let i: usize = if hh > 0xb504_f333_f9de_6484 {
        big_e += 1;
        (hh >> (63 + 1 - 7)) as usize
    } else {
        (hh >> (63 - 7)) as usize
    };
    x.ex -= big_e;

    let mut z = x.mul(&c::INVERSE_3_1[i - 90]); // exact
    let zhh = (z.hi >> 64) as u64;
    let j = (zhh >> ((63 - 13 - z.ex) as u32)) as usize;
    z = z.mul(&c::INVERSE_3_2[j - 8128]); // exact
    z = c::M_ONE_Q.add(&z); // exact

    let mut r = c::LOG2_Q.mul_int(big_e);
    let mut p = p_3(&z);
    p = c::LOG_INV_3_2[j - 8128].add(&p);
    p = c::LOG_INV_3_1[i - 90].add(&p);
    r = p.add(&r);
    r
}

/// `exp(x)` as a `Qint`, for `|x| < 744.45`, rel err < 2⁻²⁴¹·¹⁰ (port of `exp_3`).
#[inline]
fn exp_3(x: &Qint) -> Qint {
    let big_k = x.mul_11(&c::LOG2_INV_Q); // exact
    let k = big_k.to_i64();
    let mut kk = c::LOG2_Q.mul_int(k);
    kk.ex -= 12;
    kk.sgn = !kk.sgn;
    let y = x.add(&kk); // exact (Sterbenz)

    let big_m = k >> 12;
    let i2 = ((k >> 6) & 0x3f) as usize;
    let i1 = (k & 0x3f) as usize;

    let mut r = q_3(&y);
    r = c::T1_3[i2].mul(&r);
    r = c::T2_3[i1].mul(&r);
    r.ex += big_m;
    r
}

// ===========================================================================
// Exact / midpoint detection (pow.c `extract`, `pow2`, `round_54`, `exact_pow`).
// ===========================================================================

/// `x = 2^E · m` with `m` odd (port of `extract`).
#[inline]
fn extract(x: f64) -> (i64, u64) {
    let u = x.to_bits();
    let mut e = ((u >> 52) & 0x7ff) as i64;
    let mut m = (u & (!0u64 >> 12)) + if e != 0 { 1u64 << 52 } else { 0 };
    let t = m.trailing_zeros() as i64;
    m >>= t;
    e = e + t - (0x433 - i64::from(e == 0));
    (e, m)
}

/// Multiply `x` by `2^e` exactly when in range (port of `pow2`).
#[inline]
fn pow2(x: f64, e: i64) -> f64 {
    let mut x = x;
    if e & 1 != 0 {
        x *= 2.0;
    }
    let e2 = f64::from_bits(((((e >> 1) + 0x3ff) as u64) & 0x7ff) << 52);
    (x * e2) * e2
}

/// Round a `Dint` to 54 bits assuming all-ones / all-zeros tail (port of
/// `round_54`).  Returns `(G, k)` with the value rounded to `k · 2^G`.
#[inline]
fn round_54(x: &Dint) -> (i64, i64) {
    let hi = (x.m >> 64) as u64;
    let big_g = x.ex - 53;
    let k = ((hi >> 10) + ((hi >> 9) & 1)) as i64;
    (big_g, k)
}

/// Detect exact / midpoint cases (port of `exact_pow`); returns `Some(r)` with
/// the exact `f64` result when `(x, y)` is in CORE-MATH's set `S`.
///
/// `z` is the phase-2 approximation of `xʸ` (relative error < 2⁻¹¹⁷) with its
/// sign already set to the final result sign; `x` is the original (possibly
/// negative) base.  `z.sgn` therefore carries the result sign, matching the C.
#[inline]
fn exact_pow(x: f64, y: f64, z: &Dint) -> Option<f64> {
    let neg = z.sgn;
    let s_int: i64 = if neg { -1 } else { 1 };

    let (big_e, m) = extract(x.abs());

    // x is a power of 2.  `z->sgn` (= `neg`) already carries the final result
    // sign (the caller set `r.sgn = s < 0` before this call), matching the C.
    if m == 1 {
        let big_g = big_e as f64 * y;
        if is_int(big_g) {
            let r = if neg { -1.0 } else { 1.0 };
            let g = big_g as i64;
            return Some(pow2(r, g));
        }
        return None;
    }

    if y < 0.0 || y > 34.0 {
        return None;
    }

    let (big_f, n) = extract(y);
    if n > 34 || big_f < -5 {
        return None;
    }

    if big_f < 0 {
        // Case (b).
        // Check E divisible by 2^-F.
        if (big_e as u64) & (!0u64 >> ((64 + big_f) as u32)) != 0 {
            return None;
        }
        let g = (big_e >> (-big_f)) * (n as i64);

        let (big_g, k) = round_54(z);
        // Condition: if |2^G*k - z| >= 2^-116*z, reject.
        let cnt = (k as u64).leading_zeros();
        let d = Dint {
            sgn: !neg,
            ex: big_g + 63 - i64::from(cnt),
            m: u128::from((k as u64) << cnt) << 64,
        };
        let mut d = d_add(&d, z); // exact (Sterbenz)
        d.ex += 116;
        if d_cmp_abs(&d, z) != core::cmp::Ordering::Less {
            return None;
        }

        if big_g > g {
            return None;
        }

        // Check k is an odd number times 2^(g-G).
        let shift = (g - big_g) as u32;
        let k_u = k as u64;
        if (k_u & !(!1u64 << shift)) == (1u64 << shift) {
            let r = ((k_u >> shift) as i64 * s_int) as f64;
            return Some(pow2(r, g));
        }
        return None;
    }

    // Case (a): 2 <= y <= 34 integer, y = n << F.
    let t = n << big_f;
    let mut k: i64 = 1;
    let mut t_rem = t;
    let mut mm = m;
    while t_rem != 0 {
        if t_rem & 1 != 0 {
            match mm.checked_mul(k as u64) {
                Some(v) if v <= i64::MAX as u64 => k = v as i64,
                _ => return None,
            }
        }
        t_rem >>= 1;
        if t_rem != 0 {
            match mm.checked_mul(mm) {
                Some(v) => mm = v,
                None => return None,
            }
        }
    }
    if (k as u64) >> 54 != 0 {
        return None;
    }
    let r = (k * s_int) as f64;
    let big_g = big_e * (n << big_f) as i64;
    Some(pow2(r, big_g))
}

// ===========================================================================
// Top-level accurate path.
// ===========================================================================

/// Accurate `|x|ʸ · s` for finite positive `x ≠ 1`, finite `y`, `s = ±1`.
///
/// Faithful port of CORE-MATH's phase-2 → exact-detection → phase-3 cascade.
/// `x` is `|base|` (already positive); `x0` is the original (possibly negative)
/// base for [`exact_pow`]'s `2^E·m` test; `s` is the result sign.
#[inline]
pub(super) fn pow_accurate(x: f64, y: f64, x0: f64, s: f64) -> f64 {
    // --- Phase 2: dint ---
    let mut big_x = Dint::from_f64(x);
    big_x.sgn = false;
    let big_y = Dint::from_f64(y);

    let mut r = log_2(&big_x);
    r = d_mul_21(&r, &big_y);
    r = exp_2(&r);

    // Rounding test (RNDN).  metallic is RNDN-only and drops fenv/errno, so the
    // C's `exact` flag (which only gates underflow signalling) is unneeded.
    let rd = rounding_test_2(&r);

    r.sgn = s < 0.0;

    if rd {
        return d_tod(&r);
    }

    // --- Exact / midpoint detection ---
    // `r.sgn` already carries the final result sign (set just above), exactly as
    // CORE-MATH sets `R.sgn` before calling `exact_pow`.
    if let Some(e) = exact_pow(x0, y, &r) {
        return e;
    }

    // --- Phase 3: qint ---
    let mut qx = Qint::from_f64(x);
    qx.sgn = false;
    let qy = Qint::from_f64(y);

    let mut qr = log_3(&qx);
    qr = qr.mul_41(&qy);
    let mut qz = exp_3(&qr);

    // Extra rounding test.
    if rounding_test_3(&qz) {
        qz.sgn = s < 0.0;
        qz.lo &= !0u128 << 10; // clear low 10 bits of ll
        return qz.to_f64();
    }

    // xʸ very close to 1: |qR| < 2^-56.
    if qr.ex < -56 {
        return if !qr.sgn {
            1.0 + f64::from_bits(0x3990_0000_0000_0000) // 1 + 2^-100
        } else {
            1.0 - f64::from_bits(0x3990_0000_0000_0000)
        };
    }

    // Should be unreachable for valid worst cases; fall back to the dint result.
    d_tod(&r)
}

/// Phase-2 rounding test (port of the `ENABLE_ZIV2` rd computation, RNDN).
#[inline]
fn rounding_test_2(r: &Dint) -> bool {
    let hi = (r.m >> 64) as u64;
    let lo = r.m as u64;

    if r.ex < -1075 {
        return true; // underflow: rd set
    }
    if r.ex < -1022 {
        // Subnormal case.
        let ex = (-(1022 + r.ex)) as u32; // 1 <= ex <= 53
        let m = (lo >> (10 + ex)) | (hi << (54 - ex));
        return m.wrapping_add(14) > 28;
    }
    const ERR_BND_2: u64 = 28;
    let lo64 = (lo >> 10) | (hi << 54);
    lo64.wrapping_add(ERR_BND_2) > 2 * ERR_BND_2
}

/// Phase-3 rounding test (port of the `ENABLE_ZIV3` rd computation).
#[inline]
fn rounding_test_3(qz: &Qint) -> bool {
    let (hh, hl, lh, ll) = (
        (qz.hi >> 64) as u64,
        qz.hi as u64,
        (qz.lo >> 64) as u64,
        qz.lo as u64,
    );
    const ERR_BND_3: u64 = 60;
    let r1 = (hh << 54) | (hl >> 10);
    let r2 = (hl << 54) | (lh >> 10);
    let r3 = (lh << 54) | (ll >> 10);
    !((r1 == 0 && r2 == 0 && r3 <= ERR_BND_3)
        || (!r1 == 0 && !r2 == 0 && r3.wrapping_add(2 * ERR_BND_3) <= ERR_BND_3))
}

/// `true` iff `x` is an integer (round-to-even, port of `is_int`).
#[inline]
fn is_int(x: f64) -> bool {
    x == x.round_ties_even()
}
