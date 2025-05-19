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
fn test_cbrt() {
    common::test_univariate_cases(
        metal::cbrt,
        core_math::cbrt,
        common::parse_case_file("cbrt.wc", parse_f64).chain(
            (0..=u64::MAX)
                .step_by((1 << 40) - rand::random::<u32>() as usize)
                .map(f64::from_bits),
        ),
    );
}
