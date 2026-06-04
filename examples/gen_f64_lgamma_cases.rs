//! Generate `tests/cases/f64_lgamma.wc`: the hard-to-round regression corpus for
//! [`metallic::f64::lgamma`].
//!
//! `lgamma` has no correctly-rounded `f64` reference in the `core-math` crate
//! (only `lgammaf`), so MPFR is the oracle.  Each sampled `z` is evaluated at
//! [`PREC`] bits; we keep the tiny fraction whose true `ln|Γ(z)|` lands within
//! [`THRESHOLD`] of an `f64` midpoint (normalized so 0 is a midpoint and 1 a grid
//! point) and emit each survivor with its correctly-rounded result.  `THRESHOLD`
//! is far wider than the implementation's real danger band (≈2⁻⁴⁵ normalized), so
//! the corpus guards against any regression that weakens the rounding well before
//! it could flip a bit.  The scan is embarrassingly parallel and deterministic.
//!
//! Run with:
//! ```text
//! cargo run --release --features mpfr --example gen_f64_lgamma_cases
//! ```

use rug::Float;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

/// Working precision for the MPFR oracle (250 bits leaves ~190 below the `f64`
/// ulp even at the overflow end, so `to_f64` is correctly rounded everywhere).
const PREC: u32 = 250;

/// Keep results within this *normalized* distance of an `f64` midpoint.
const THRESHOLD: f64 = 1.0 / 1_048_576.0; // 2⁻²⁰

/// Inputs scanned per family.
const COUNT: u64 = 400_000_000;

/// SplitMix64 hash.
fn mix(i: u64) -> u64 {
    let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// `z` for index `i`: a hashed mantissa with exponent in `[2⁻³⁰, 2⁸)`, signed by
/// `neg`.  Out-of-domain and near-integer-pole samples are rejected in [`scan`].
fn zval(i: u64, neg: bool) -> f64 {
    let h = mix(i ^ if neg { 0xF00D_BABE_1234_5678 } else { 0 });
    let exp = 1023 - 30 + (h >> 52) % 38; // unbiased exponent in [-30, 7]
    let v = f64::from_bits((exp << 52) | (h & 0x000F_FFFF_FFFF_FFFF));
    if neg { -v } else { v }
}

/// Normalized distance from `y` to the nearest `f64` midpoint (1 = grid point).
fn midpoint_frac(y: &Float) -> f64 {
    let c = y.to_f64();
    if !c.is_finite() || c == 0.0 {
        return 1.0;
    }
    let mid_hi = (Float::with_val(PREC, c) + Float::with_val(PREC, c.next_up())) / 2u32;
    let mid_lo = (Float::with_val(PREC, c) + Float::with_val(PREC, c.next_down())) / 2u32;
    let d_hi = Float::with_val(PREC, y - &mid_hi).abs();
    let d_lo = Float::with_val(PREC, y - &mid_lo).abs();
    if d_hi < d_lo {
        (d_hi / (mid_hi - c)).to_f64()
    } else {
        (d_lo / (c - mid_lo)).to_f64()
    }
}

/// `ln|Γ(z)|` at [`PREC`] bits.
fn oracle(z: f64) -> Float {
    Float::with_val(PREC, z).ln_abs_gamma().0
}

/// Within `2⁻²⁰` of a non-positive integer (pole or exact-handled), skip.
fn near_pole(z: f64) -> bool {
    z < 0.5 && (z - z.round()).abs() < THRESHOLD
}

/// Exact C99 hexfloat for a finite `f64`.
fn hexf64(x: f64) -> String {
    let bits = x.to_bits();
    let sign = if bits >> 63 == 1 { "-" } else { "" };
    let exp = ((bits >> 52) & 0x7FF) as i32;
    let frac = bits & 0x000F_FFFF_FFFF_FFFF;
    let s = match exp {
        0 if frac == 0 => format!("{sign}0x0p+0"),
        0 => format!("{sign}0x0.{frac:013x}p-1022"),
        _ => format!("{sign}0x1.{frac:013x}p{:+}", exp - 1023),
    };
    assert_eq!(
        hexf_parse::parse_hexf64(&s, false).unwrap().to_bits(),
        bits,
        "hexf64 round-trip failed for {x:e}"
    );
    s
}

/// Scan one family in parallel, returning the surviving `z`.
fn scan(name: &str, neg: bool) -> Vec<f64> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let chunk = COUNT.div_ceil(threads as u64);
    let done = AtomicU64::new(0);

    let mut found: Vec<f64> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads as u64)
            .map(|t| {
                let done = &done;
                scope.spawn(move || {
                    let lo = t * chunk;
                    let hi = (lo + chunk).min(COUNT);
                    let mut local = Vec::new();
                    for i in lo..hi {
                        let z = zval(i, neg);
                        if near_pole(z) {
                            continue;
                        }
                        if midpoint_frac(&oracle(z)) < THRESHOLD {
                            local.push(z);
                        }
                        if i % (1 << 24) == 0 {
                            let n = done.fetch_add(1 << 24, Ordering::Relaxed) + (1 << 24);
                            eprint!("\r{name}: {n} / {COUNT} scanned, {} found ", local.len());
                        }
                    }
                    local
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    eprintln!("\r{name}: {COUNT} scanned, {} found{:20}", found.len(), "");
    found.sort_unstable_by_key(|z| z.to_bits());
    found
}

fn main() {
    let mut cases: Vec<f64> = scan("pos", false);
    cases.extend(scan("neg", true));
    cases.sort_unstable_by_key(|z| z.to_bits());
    cases.dedup_by_key(|z| z.to_bits());

    let mut out = String::new();
    out.push_str("# Hard-to-round cases for metallic::f64::lgamma(z).\n");
    out.push_str("# Columns: z, correctly-rounded lgamma(z).\n");
    out.push_str(
        "# Generated by `cargo run --release --features mpfr --example gen_f64_lgamma_cases`.\n",
    );

    let mut mismatches = 0;
    for &z in &cases {
        let want = oracle(z).to_f64();
        let got = metallic::f64::lgamma(z);
        if got.to_bits() != want.to_bits() {
            mismatches += 1;
            eprintln!("MISMATCH lgamma({z:e}) = {got:e} != {want:e} (correct)");
        }
        writeln!(out, "{}, {}", hexf64(z), hexf64(want)).unwrap();
    }

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cases/f64_lgamma.wc");
    std::fs::write(path, out).unwrap();
    eprintln!(
        "wrote {} cases to {path} ({mismatches} current mismatches)",
        cases.len(),
    );
    assert_eq!(
        mismatches, 0,
        "metallic::f64::lgamma is not correctly rounded"
    );
}
