#![doc = include_str!("../README.md")]

use core::ops::Neg;
use clap::{Parser, Subcommand};
use hexf_parse::ParseHexfError;
use regex::Regex;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Subcommand to specify which type for output
#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    /// Output binary is in [`f32`] format
    F32,
    /// Output binary is in [`f64`] format
    F64,
}

/// Command line arguments
#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(version, about)]
struct Args {
    /// Output binary file
    output: PathBuf,

    /// Textual `.wc` to read
    #[arg(short, long, value_name = "FILE")]
    input: Option<PathBuf>,

    /// Subcommand to specify which type for output
    #[command(subcommand)]
    command: Command,
}

/// Parse a string as a [`f32`]
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
                Some(b'-') => fallback(&s[1..]).map(Neg::neg),
                _ => fallback(s),
            }
            .ok_or(e)
        }),
    }
}

/// Parse a string as a [`f64`]
fn parse_f64(s: &str) -> Result<f64, ParseHexfError> {
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
                Some(b'-') => fallback(&s[1..]).map(Neg::neg),
                _ => fallback(s),
            }
            .ok_or(e)
        }),
    }
}

/// Convert CSV to binary
///
/// - Whitespace is ignored
/// - Lines starting with `#` are ignored
/// - Each line is split by `,`
/// - Each field is parsed by `kernel` and produces `N` bytes
fn convert<const N: usize>(
    kernel: impl Fn(&str) -> anyhow::Result<[u8; N]>,
    mut writer: impl io::Write,
    stream: impl io::BufRead,
) -> anyhow::Result<()> {
    static SEPARATOR: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\s*,\s*").expect("Failed to compile SEPARATOR"));

    for line in stream.lines() {
        let line = line?;
        let line = line.trim_ascii_start();

        if matches!(line.bytes().next(), Some(b'#') | None) {
            continue;
        }
        for s in SEPARATOR.split(line) {
            writer.write_all(kernel(s)?.as_ref())?;
        }
    }
    Ok(())
}

/// Select callback from `command`
fn select(
    command: Command,
    writer: impl io::Write,
    stream: impl io::BufRead,
) -> anyhow::Result<()> {
    match command {
        Command::F32 => convert(|s| Ok(parse_f32(s).map(f32::to_le_bytes)?), writer, stream),
        Command::F64 => convert(|s| Ok(parse_f64(s).map(f64::to_le_bytes)?), writer, stream),
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let writer = io::BufWriter::new(File::create(args.output)?);

    match args.input {
        Some(path) => select(args.command, writer, io::BufReader::new(File::open(path)?)),
        None => select(args.command, writer, io::stdin().lock()),
    }
}
