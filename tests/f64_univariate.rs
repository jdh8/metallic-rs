mod common;
use metallic::f64 as metal;
use std::io::BufRead as _;
use std::path::PathBuf;

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

fn parse_cases_from<T, E>(
    filename: impl AsRef<std::ffi::OsStr>,
    mut parse: impl FnMut(&str) -> Result<T, E>,
) -> impl Iterator<Item = T> {
    let path: PathBuf = file!().into();
    let path = path.with_file_name(filename);

    std::fs::File::open(path)
        .map(std::io::BufReader::new)
        .map(|stream| {
            stream
                .lines()
                .map_while(Result::ok)
                .filter_map(move |line| {
                    let line = line[..line.find('#').unwrap_or(line.len())].trim_ascii();
                    parse(line).ok()
                })
        })
        .into_iter()
        .flatten()
}

#[test]
fn test_parser() {
    assert_eq!(parse_cases_from("worst-cases/cbrt.wc", parse_f64).count(), 105_554);
}

#[test]
fn test_cbrt() {
    common::test_univariate_cases(
        metal::cbrt,
        core_math::cbrt,
        parse_cases_from("worst-cases/cbrt.wc", parse_f64),
    );
}
