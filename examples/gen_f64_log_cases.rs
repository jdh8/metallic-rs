//! Generate `tests/cases/f64_log.wc`: the hard-to-round regression corpus for
//! [`metallic::f64::log`].
//!
//! `metallic::f64::log` computes `log2_dd(x) / log2_dd(base)` in double-double
//! (≈2⁻⁹⁴) and rounds once, so it mis-rounds only when the true result sits
//! within ~2⁻⁹⁴ (relative) of an `f64` midpoint — an absurdly narrow band.
//! Unlike the `f32` generator, a plain `f64` proxy is useless here: its ~2⁻⁵²
//! error already exceeds an `f64` ulp, so it cannot even locate a midpoint.
//!
//! We therefore use MPFR itself as the proxy.  Each `(x, base)` is evaluated at
//! [`PREC`] bits; we keep the tiny fraction whose true value lands within
//! [`THRESHOLD`] of an `f64` midpoint (normalized so 0 is on a midpoint and 1 is
//! on a grid point) and emit each survivor with its correctly-rounded result.
//! `THRESHOLD` is far wider than the real ≈2⁻⁴¹ (normalized) danger band, so the
//! corpus catches any regression that weakens the rounding well before it could
//! produce a wrong bit, while staying small enough to commit.  The scan is
//! embarrassingly parallel; each input is a pure function of its index, so the
//! output is deterministic regardless of thread scheduling.
//!
//! Run with:
//! ```text
//! cargo run --release --features mpfr --example gen_f64_log_cases
//! ```

use rug::Float;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

/// Working precision for the MPFR oracle.  `ln(x)/ln(base)` at 256 bits, rounded
/// once to `f64`, leaves ~203 bits below the `f64` ulp — a correctly-rounded
/// result for every input here (the same trust the `f32` generator places in its
/// 200-bit oracle, with even more headroom).
const PREC: u32 = 256;

/// Keep results within this *normalized* distance of an `f64` midpoint (0 =
/// midpoint, 1 = grid point).  2⁻²⁰ is ~2²¹ wider than the implementation's real
/// danger band, leaving a large margin while bounding the corpus size.
const THRESHOLD: f64 = 1.0 / 1_048_576.0; // 2⁻²⁰

/// Inputs scanned per family.  Survivors ≈ `WIDE` (resp. `NEAR1`) × `THRESHOLD`.
const WIDE: u64 = 2_000_000_000;
const NEAR1: u64 = 1_000_000_000;

/// SplitMix64: turn a counter into well-distributed bits, avoiding the
/// hyperplane lattice a plain multiplicative hash would leave in (x, base).
fn mix(i: u64) -> u64 {
    let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Finite positive normal `f64` from a hash: exponent in `[1, 2046]`, hashed
/// 52-bit mantissa.
fn posf(hash: u64) -> f64 {
    let exp = 1 + (hash >> 52) % 2046;
    let mant = hash & 0x000F_FFFF_FFFF_FFFF;
    f64::from_bits((exp << 52) | mant)
}

/// The `(x, base)` pair for index `i` of the given family.
fn wide_pair(i: u64) -> [f64; 2] {
    [posf(mix(i)), posf(mix(i ^ 0x9E37_79B9_7F4A_7C15))]
}

/// base near 1 (large results — the hardest rounding), `x, base ∈ [1, 2)`.
fn near1_pair(i: u64) -> [f64; 2] {
    let hx = mix(i ^ 0xDEAD_BEEF_CAFE_F00D);
    let hb = mix(i ^ 0x1234_5678_9ABC_DEF0);
    let x = f64::from_bits(0x3FF0_0000_0000_0000 | (hx >> 12));
    // base mantissa drawn from the top 50 bits keeps base ∈ [1, 1.25), so
    // `log2(base)` is small and results run large.
    let base = f64::from_bits(0x3FF0_0000_0000_0000 | (hb >> 14));
    [x, base]
}

/// Normalized distance from the true value `y` to the nearest `f64` midpoint, in
/// MPFR so the result is exact to far below [`THRESHOLD`].  Returns 1 (a grid
/// point, never collected) for non-finite roundings.
fn midpoint_frac(y: &Float) -> f64 {
    let c = y.to_f64();
    if !c.is_finite() {
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

/// `ln(x)/ln(base)` at [`PREC`] bits.
fn oracle(x: f64, base: f64) -> Float {
    let lx = Float::with_val(PREC, x).ln();
    let lb = Float::with_val(PREC, base).ln();
    lx / lb
}

/// Exact C99 hexfloat for a finite `f64`, matching the existing `.wc` corpora.
fn hexf64(x: f64) -> String {
    let bits = x.to_bits();
    let sign = if bits >> 63 == 1 { "-" } else { "" };
    let exp = ((bits >> 52) & 0x7FF) as i32;
    let frac = bits & 0x000F_FFFF_FFFF_FFFF; // 52-bit mantissa = 13 hex digits
    let s = match exp {
        0 if frac == 0 => format!("{sign}0x0p+0"),
        0 => format!("{sign}0x0.{frac:013x}p-1022"), // subnormal
        _ => format!("{sign}0x1.{frac:013x}p{:+}", exp - 1023),
    };
    assert_eq!(
        hexf_parse::parse_hexf64(&s, false).unwrap().to_bits(),
        bits,
        "hexf64 round-trip failed for {x:e}"
    );
    s
}

/// Scan one family in parallel, returning the surviving `(x, base)` pairs.
fn scan(name: &str, count: u64, pair: impl Fn(u64) -> [f64; 2] + Sync) -> Vec<[f64; 2]> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let chunk = count.div_ceil(threads as u64);
    let done = AtomicU64::new(0);

    let mut found: Vec<[f64; 2]> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads as u64)
            .map(|t| {
                let pair = &pair;
                let done = &done;
                scope.spawn(move || {
                    let lo = t * chunk;
                    let hi = (lo + chunk).min(count);
                    let mut local = Vec::new();
                    for i in lo..hi {
                        let [x, base] = pair(i);
                        if x == 1.0 || base == 1.0 {
                            continue;
                        }
                        if midpoint_frac(&oracle(x, base)) < THRESHOLD {
                            local.push([x, base]);
                        }
                        if i % (1 << 24) == 0 {
                            let n = done.fetch_add(1 << 24, Ordering::Relaxed) + (1 << 24);
                            eprint!("\r{name}: {n} / {count} scanned, {} found ", local.len());
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
    eprintln!("\r{name}: {count} scanned, {} found{:20}", found.len(), "");
    found.sort_unstable_by_key(|&[x, base]| (x.to_bits(), base.to_bits()));
    found
}

fn main() {
    let mut cases: Vec<[f64; 2]> = scan("wide", WIDE, wide_pair);
    cases.extend(scan("near1", NEAR1, near1_pair));
    cases.sort_unstable_by_key(|&[x, base]| (x.to_bits(), base.to_bits()));
    cases.dedup_by_key(|&mut [x, base]| (x.to_bits(), base.to_bits()));

    let mut out = String::new();
    out.push_str("# Hard-to-round cases for metallic::f64::log(x, base).\n");
    out.push_str("# Columns: x, base, correctly-rounded log(x, base).\n");
    out.push_str(
        "# Generated by `cargo run --release --features mpfr --example gen_f64_log_cases`.\n",
    );

    let mut mismatches = 0;
    for &[x, base] in &cases {
        let want = oracle(x, base).to_f64();
        let got = metallic::f64::log(x, base);
        if got.to_bits() != want.to_bits() {
            mismatches += 1;
            eprintln!("MISMATCH log({x:e}, {base:e}) = {got:e} != {want:e} (correct)");
        }
        writeln!(out, "{}, {}, {}", hexf64(x), hexf64(base), hexf64(want)).unwrap();
    }

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cases/f64_log.wc");
    std::fs::write(path, out).unwrap();
    eprintln!(
        "wrote {} cases to {path} ({mismatches} current mismatches)",
        cases.len(),
    );
    assert_eq!(mismatches, 0, "metallic::f64::log is not correctly rounded");
}
