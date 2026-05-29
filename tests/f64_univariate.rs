mod common;
use metallic::f64 as metal;

fn parse_f64(s: &str) -> Result<f64, hexf_parse::ParseHexfError> {
    fn fallback(s: &str) -> Option<f64> {
        match s {
            "snan" => Some(f64::from_bits(f64::NAN.to_bits() | 1)),
            #[allow(clippy::cast_precision_loss)]
            s if s.starts_with("0x") => u64::from_str_radix(&s[2..], 16).ok().map(|x| x as f64),
            _ => None,
        }
    }

    match hexf_parse::parse_hexf64(s, true) {
        Ok(value) => Ok(value),
        Err(e) => s.parse().or_else(|_| {
            match s.bytes().next() {
                Some(b'+') => fallback(&s[1..]),
                Some(b'-') => fallback(&s[1..]).map(core::ops::Neg::neg),
                _ => fallback(s),
            }
            .ok_or(e)
        }),
    }
}

#[test]
fn test_parser() {
    assert!(common::parse_case_file("cbrt.wc", parse_f64).count() == 105_554);
}

#[test]
fn test_exp() {
    // Dense sweep of the finite range [−745, 710] plus bit-pattern stepping over
    // all of `f64` (covers ±0, subnormals, overflow/underflow, NaN, ∞).
    let dense = (0..=2_000_000).map(|i| -745.2 + f64::from(i) * (1455.0 / 2_000_000.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metal::exp, core_math::exp, dense.chain(bits));
}

#[test]
fn test_exp2() {
    let dense = (0..=2_000_000).map(|i| -1075.0 + f64::from(i) * (2099.0 / 2_000_000.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metal::exp2, core_math::exp2, dense.chain(bits));
}

#[test]
fn test_exp10() {
    let dense = (0..=2_000_000).map(|i| -323.7 + f64::from(i) * (632.0 / 2_000_000.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metal::exp10, core_math::exp10, dense.chain(bits));
}

#[test]
fn test_exp_m1() {
    let dense = (0..=2_000_000).map(|i| -710.0 + f64::from(i) * (1420.0 / 2_000_000.0));
    let bits = (0..=u64::MAX).step_by((1 << 38) - 1337).map(f64::from_bits);
    common::test_univariate_cases(metal::exp_m1, core_math::expm1, dense.chain(bits));
}

/// Bit-pattern sweep plus a dense sweep of [0.5, 2] (the cancellation region near 1).
fn log_inputs() -> impl Iterator<Item = f64> {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    let near_one = (0..=2_000_000).map(|i| 0.5 + f64::from(i) * (1.5 / 2_000_000.0));
    bits.chain(near_one)
}

#[test]
fn test_ln() {
    common::test_univariate_cases(metal::ln, core_math::log, log_inputs());
}

#[test]
fn test_log2() {
    common::test_univariate_cases(metal::log2, core_math::log2, log_inputs());
}

#[test]
fn test_log10() {
    common::test_univariate_cases(metal::log10, core_math::log10, log_inputs());
}

#[test]
fn test_ln_1p() {
    let bits = (0..=u64::MAX).step_by((1 << 37) - 1337).map(f64::from_bits);
    let near_zero = (0..=2_000_000).map(|i| -0.5 + f64::from(i) * (1.5 / 2_000_000.0));
    common::test_univariate_cases(metal::ln_1p, core_math::log1p, bits.chain(near_zero));
}

#[test]
fn test_cbrt() {
    common::test_univariate_cases(
        metal::cbrt,
        core_math::cbrt,
        common::parse_case_file("cbrt.wc", parse_f64)
            .chain((0..=u64::MAX).step_by((1 << 40) - 1337).map(f64::from_bits)),
    );
}
