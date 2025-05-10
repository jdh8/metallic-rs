use metallic::f64 as metal;
use std::io::BufRead as _;
use std::path::PathBuf;

/// Semantic identity like `Object.is` in JavaScript
///
/// This function works around comparison issues with NaNs and signed zeros.
/// To be specific, `is(f64::NAN, f64::NAN)` but not `is(0.0, -0.0)`.
trait Identity {
    fn is(&self, other: &Self) -> bool;
}

impl Identity for f64 {
    fn is(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits() || (self.is_nan() && other.is_nan())
    }
}

impl<T: Identity, U: Identity> Identity for (T, U) {
    fn is(&self, other: &Self) -> bool {
        self.0.is(&other.0) && self.1.is(&other.1)
    }
}

/// Check if `f` returns the same result as `g` for the worse cases
///
/// By "same result", I mean semantic identity as defined by [`is`].
fn test_worst_cases<T: Identity + core::fmt::Debug>(
    f: impl Fn(f64) -> T,
    g: impl Fn(f64) -> T,
    cases: impl Iterator<Item = f64>,
) {
    const LIMIT: usize = 250;

    let count = cases
        .filter_map(|x| {
            let f = f(x);
            let g = g(x);
            (!f.is(&g)).then(|| println!("{x:e}: {f:?} != {g:?}"))
        })
        .take(LIMIT)
        .count();

    assert!(
        count < LIMIT,
        "Too many (>= {LIMIT}) mismatches!  Aborting...",
    );
    assert!(count == 0, "There are {count} mismatches");
}

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
    assert_eq!(parse_cases_from("cbrt.wc", parse_f64).count(), 105_554);
}

#[test]
fn test_cbrt() {
    test_worst_cases(
        metal::cbrt,
        core_math::cbrt,
        parse_cases_from("cbrt.wc", parse_f64),
    );
}
