use metallic::f32 as metal;
use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;

fn parse_f32(s: &str) -> Result<f32, hexf_parse::ParseHexfError> {
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

fn parse_pairs(stream: impl std::io::BufRead) -> impl Iterator<Item = [f32; 2]> {
    static SEPARATOR: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\s*,\s*").expect("Failed to compile SEPARATOR"));

    stream.lines().map_while(Result::ok).filter_map(|line| {
        let line = line[..line.find('#').unwrap_or(line.len())].trim_ascii();
        let mut fields = SEPARATOR.splitn(line, 2);
        let x = fields.next().and_then(|s| parse_f32(s).ok())?;
        let y = fields.next().and_then(|s| parse_f32(s).ok())?;
        Some([x, y])
    })
}

fn parse_pairs_from(filename: impl AsRef<std::ffi::OsStr>) -> impl Iterator<Item = [f32; 2]> {
    let path: PathBuf = file!().into();
    let path = path.with_file_name(filename);

    std::fs::File::open(path)
        .map(std::io::BufReader::new)
        .map(parse_pairs)
        .into_iter()
        .flatten()
}

#[test]
fn test_parser() {
    assert!(parse_pairs_from("hypotf.wc").count() == 6882);
    assert!(parse_pairs_from("powf.wc").count() == 133_216);
}

#[test]
fn test_hypot() {
    parse_pairs_from("hypotf.wc").for_each(|[x, y]| {
        assert!(super::is(metal::hypot(x, y), core_math::hypotf(x, y)));
    });
}
