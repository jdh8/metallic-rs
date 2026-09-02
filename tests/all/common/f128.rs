#![allow(dead_code)]

use crate::common;
use common::Identity as _;

impl common::Identity for f128 {
    #[inline]
    fn is(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits() || (self.is_nan() && other.is_nan())
    }
}

/// ULP distance for binary128's full sign-magnitude representation space.
pub fn ulp_error_f128(a: f128, b: f128) -> u128 {
    const SIGN: u128 = 1 << 127;

    if a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan()) {
        return 0;
    }
    if a.is_nan() || b.is_nan() {
        return u128::MAX;
    }
    let ordered = |x: f128| {
        let bits = x.to_bits();
        bits ^ (((bits as i128) >> 127) as u128 | SIGN)
    };
    ordered(a).abs_diff(ordered(b))
}

/// Two independent SplitMix64 words, for deterministic `f128` bit sampling.
pub fn mix128(i: u64) -> u128 {
    u128::from(common::mix64(i)) | (u128::from(common::mix64(i ^ 0x9E37_79B9_7F4A_7C15)) << 64)
}

/// CORE-MATH binary128 worst-case gate.
pub fn test_worst_univariate_f128(
    name: &str,
    f: impl Fn(f128) -> f128,
    oracle: impl Fn(f128) -> f128,
) {
    common::test_univariate_cases(
        f,
        oracle,
        common::parse_case_file(format!("{name}q.wc"), parse_f128),
    );
}

/// [`common::test_bivariate_cases`] for `f128`, which has no `LowerExp`.
pub fn test_bivariate_cases_f128(
    f: impl Fn(f128, f128) -> f128,
    g: impl Fn(f128, f128) -> f128,
    cases: impl Iterator<Item = [f128; 2]>,
) {
    common::truncate_errors(cases.filter_map(|[x, y]| {
        let (f, g) = (f(x, y), g(x, y));
        (!f.is(&g)).then(|| println!("{x:?}, {y:?}: {f:?} != {g:?}"))
    }));
}

/// Bivariate [`test_worst_univariate_f128`].
pub fn test_worst_bivariate_f128(
    name: &str,
    f: impl Fn(f128, f128) -> f128,
    oracle: impl Fn(f128, f128) -> f128,
) {
    test_bivariate_cases_f128(
        f,
        oracle,
        common::parse_case_file(format!("{name}q.wc"), parse_f128_pair),
    );
}

/// MPFR sweep for a binary128 univariate function.
#[cfg(feature = "mpfr")]
pub fn mpfr_sweep_univariate_f128(
    f: impl Fn(f128) -> f128,
    cr: impl Fn(f128) -> f128,
    sampler: impl Fn(u64) -> f128,
    n: u64,
) {
    common::test_univariate_cases(f, cr, (0..n).map(sampler));
}

/// Bivariate [`mpfr_sweep_univariate_f128`].
#[cfg(feature = "mpfr")]
pub fn mpfr_sweep_bivariate_f128(
    f: impl Fn(f128, f128) -> f128,
    cr: impl Fn(f128, f128) -> f128,
    sampler: impl Fn(u64) -> [f128; 2],
    n: u64,
) {
    test_bivariate_cases_f128(f, cr, (0..n).map(sampler));
}

#[derive(Debug)]
pub struct ParseF128Error;

/// Parse a whitespace-separated `f128` pair: CORE-MATH's binary128 corpora put
/// a space where the binary64 ones put a comma.
pub fn parse_f128_pair(s: &str) -> Result<[f128; 2], ParseF128Error> {
    let mut fields = s.split_ascii_whitespace();
    let x = parse_f128(fields.next().ok_or(ParseF128Error)?)?;
    let y = parse_f128(fields.next().ok_or(ParseF128Error)?)?;
    Ok([x, y])
}

/// Parse a whitespace-separated `f128` triple: two arguments and the answer.
pub fn parse_f128_triple(s: &str) -> Result<[f128; 3], ParseF128Error> {
    let mut fields = s.split_ascii_whitespace();
    let x = parse_f128(fields.next().ok_or(ParseF128Error)?)?;
    let y = parse_f128(fields.next().ok_or(ParseF128Error)?)?;
    let z = parse_f128(fields.next().ok_or(ParseF128Error)?)?;
    Ok([x, y, z])
}

/// Parse the decimal integers, specials, and hexadecimal floats used by
/// CORE-MATH's binary128 corpora, with round-to-nearest-even conversion.
pub fn parse_f128(s: &str) -> Result<f128, ParseF128Error> {
    const SIGN: u128 = 1 << 127;
    const FRAC_BITS: i64 = 112;
    const MIN_NORMAL_EXP: i64 = -16_382;
    const MIN_SUBNORMAL_EXP: i64 = -16_494;
    const MAX_EXP: i64 = 16_383;
    const BIAS: i64 = 16_383;

    let (negative, body) = match s.as_bytes().first() {
        Some(b'+') => (false, &s[1..]),
        Some(b'-') => (true, &s[1..]),
        Some(_) => (false, s),
        None => return Err(ParseF128Error),
    };
    let sign = if negative { SIGN } else { 0 };

    let special = match body {
        "inf" => Some(f128::INFINITY.to_bits()),
        "nan" | "qnan" => Some(f128::NAN.to_bits()),
        "snan" => Some(f128::NAN.to_bits() | 1),
        _ => None,
    };
    if let Some(bits) = special {
        return Ok(f128::from_bits(sign | bits));
    }

    if !body.starts_with("0x") {
        let value = body.parse::<u128>().map_err(|_| ParseF128Error)? as f128;
        return Ok(f128::from_bits(sign | value.to_bits()));
    }

    let (mantissa, exponent) = body[2..].split_once(['p', 'P']).ok_or(ParseF128Error)?;
    let exponent = exponent.parse::<i64>().map_err(|_| ParseF128Error)?;
    let (integer, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if integer.is_empty() && fraction.is_empty() {
        return Err(ParseF128Error);
    }

    let mut digits = String::with_capacity(integer.len() + fraction.len());
    digits.push_str(integer);
    digits.push_str(fraction);
    if !digits.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(ParseF128Error);
    }
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return Ok(f128::from_bits(sign));
    }

    let first = (digits.as_bytes()[0] as char)
        .to_digit(16)
        .ok_or(ParseF128Error)?;
    let bit_len = digits
        .len()
        .checked_sub(1)
        .and_then(|n| n.checked_mul(4))
        .and_then(|n| i64::try_from(n).ok())
        .ok_or(ParseF128Error)?
        + i64::from(32 - first.leading_zeros());
    let fraction_len = i64::try_from(fraction.len()).map_err(|_| ParseF128Error)?;
    let scale = exponent
        .checked_sub(fraction_len.checked_mul(4).ok_or(ParseF128Error)?)
        .ok_or(ParseF128Error)?;
    let mut value_exp = scale.checked_add(bit_len - 1).ok_or(ParseF128Error)?;
    if value_exp > MAX_EXP {
        return Ok(f128::from_bits(sign | f128::INFINITY.to_bits()));
    }

    // 32 nibbles fit in `u128`. Lower discarded nibbles contribute only a
    // sticky bit; 128 retained bits leave at least 15 guard bits for binary128.
    let keep = digits.len().min(32);
    let significand = u128::from_str_radix(&digits[..keep], 16).map_err(|_| ParseF128Error)?;
    let dropped = digits.len() - keep;
    let sticky = digits[keep..].bytes().any(|c| c != b'0');
    let retained_scale = scale
        .checked_add(
            i64::try_from(dropped)
                .map_err(|_| ParseF128Error)?
                .checked_mul(4)
                .ok_or(ParseF128Error)?,
        )
        .ok_or(ParseF128Error)?;
    let quantum = if value_exp >= MIN_NORMAL_EXP {
        value_exp - FRAC_BITS
    } else {
        MIN_SUBNORMAL_EXP
    };
    let shift = quantum.checked_sub(retained_scale).ok_or(ParseF128Error)?;
    let mut rounded = if shift <= 0 {
        if sticky {
            return Err(ParseF128Error);
        }
        significand
            .checked_shl(u32::try_from(-shift).map_err(|_| ParseF128Error)?)
            .ok_or(ParseF128Error)?
    } else if shift < 128 {
        let shift = u32::try_from(shift).map_err(|_| ParseF128Error)?;
        let truncated = significand >> shift;
        let remainder = significand & ((1u128 << shift) - 1);
        let halfway = 1u128 << (shift - 1);
        truncated
            + u128::from(
                remainder > halfway || (remainder == halfway && (sticky || truncated & 1 != 0)),
            )
    } else if shift == 128 {
        u128::from(significand > 1 << 127 || (significand == 1 << 127 && sticky))
    } else {
        0
    };

    if value_exp < MIN_NORMAL_EXP {
        return Ok(f128::from_bits(sign | rounded));
    }
    if rounded == 1 << 113 {
        rounded >>= 1;
        value_exp += 1;
        if value_exp > MAX_EXP {
            return Ok(f128::from_bits(sign | f128::INFINITY.to_bits()));
        }
    }
    let biased = u128::try_from(value_exp + BIAS).map_err(|_| ParseF128Error)?;
    Ok(f128::from_bits(
        sign | (biased << 112) | (rounded - (1 << 112)),
    ))
}
