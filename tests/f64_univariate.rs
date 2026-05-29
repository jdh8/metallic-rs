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
fn test_cbrt() {
    common::test_univariate_cases(
        metal::cbrt,
        core_math::cbrt,
        common::parse_case_file("cbrt.wc", parse_f64)
            .chain((0..=u64::MAX).step_by((1 << 40) - 1337).map(f64::from_bits)),
    );
}
