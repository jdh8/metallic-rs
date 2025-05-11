mod common;
use common::Identity as _;
use hexf_parse::ParseHexfError;
use metallic::f32 as metal;

enum ParsePairError {
    EmptyField,
    Hexf,
}

impl From<ParseHexfError> for ParsePairError {
    fn from(_: ParseHexfError) -> Self {
        Self::Hexf
    }
}

fn parse_f32(s: &str) -> Result<f32, ParseHexfError> {
    fn fallback(s: &str) -> Option<f32> {
        match s {
            "snan" => Some(f32::from_bits(f32::NAN.to_bits() | 1)),
            #[allow(clippy::cast_precision_loss)]
            s if s.starts_with("0x") => u32::from_str_radix(&s[2..], 16).ok().map(|x| x as f32),
            _ => None,
        }
    }

    match hexf_parse::parse_hexf32(s, true) {
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

fn parse_f32_pair(s: &str) -> Result<[f32; 2], ParsePairError> {
    let mut fields = s.splitn(2, ',').map(str::trim_ascii);
    let x = parse_f32(fields.next().ok_or(ParsePairError::EmptyField)?)?;
    let y = parse_f32(fields.next().ok_or(ParsePairError::EmptyField)?)?;
    Ok([x, y])
}

fn test_bivariate(
    f: impl Fn(f32, f32) -> f32,
    g: impl Fn(f32, f32) -> f32,
    cases: impl Iterator<Item = [f32; 2]>,
) {
    common::truncate_errors(
        cases
            .filter(|&[x, y]| (!f(x, y).is(&g(x, y))))
            .map(|[x, y]| println!("{x:e}, {y:e}: {:e} != {:e}", f(x, y), g(x, y))),
    );
}

#[test]
fn test_parser() {
    assert!(common::parse_case_file("hypotf.wc", parse_f32_pair).count() == 6882);
    assert!(common::parse_case_file("powf.wc", parse_f32_pair).count() == 133_216);
}

#[test]
fn test_hypot() {
    test_bivariate(
        metal::hypot,
        core_math::hypotf,
        common::parse_case_file("hypotf.wc", parse_f32_pair),
    );
}

#[test]
// Signal when precision improves
#[should_panic = "Too many (>= 250) mismatches!  Aborting..."]
fn test_powf() {
    test_bivariate(
        metal::powf,
        core_math::powf,
        common::parse_case_file("powf.wc", parse_f32_pair),
    );
}
