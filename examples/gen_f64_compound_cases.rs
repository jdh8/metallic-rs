//! Generate `tests/cases/f64_compound.wc`: the hard-to-round regression corpus
//! for [`metallic::compound`].
//!
//! `compound` has no correctly-rounded `f64` reference in the `core-math` crate
//! (only `compoundf`), so MPFR is the oracle.  The corpus concentrates on the
//! *novel* region — small `|x|`, where `1 + x` is inexact and the correctly-
//! rounded `pow(1+x, y)` cross-check (see `tests/compound.rs`) cannot reach.
//! Each sampled `(x, y)` evaluates `(1 + x)ʸ` at [`PREC`] bits (the `1 + x` sum
//! is exact there for every finite `f64` `x`); we keep the tiny fraction whose
//! true value lands within [`THRESHOLD`] of an `f64` midpoint and emit each
//! survivor with its correctly-rounded result.  The scan is embarrassingly
//! parallel and deterministic.
//!
//! Run with:
//! ```text
//! cargo run --release --features mpfr --example gen_f64_compound_cases
//! ```

use rug::Float;
use rug::ops::Pow;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

/// Working precision for the MPFR oracle (256 bits leaves ~190 below the `f64`
/// ulp across the whole exponent range, so `to_f64` is correctly rounded, and
/// `1 + x` is exact for every finite `f64` `x`).
const PREC: u32 = 256;

/// Keep results within this *normalized* distance of an `f64` midpoint.  At 2⁻¹⁸
/// the band is ~2²⁷ wider than the implementation's real danger (≈2⁻⁴⁵), so every
/// survivor is a genuine near-miss while a cheap scan reaches a dense corpus.
const THRESHOLD: f64 = 1.0 / 262_144.0; // 2⁻¹⁸

/// Inputs scanned (survivors ≈ `COUNT` × `THRESHOLD` × in-range-rate).
const COUNT: u64 = 800_000_000;

/// `log₂ e`, to place a target result exponent via `y ≈ E / (log₂ e · ln(1+x))`.
const LOG2_E: f64 = std::f64::consts::LOG2_E;

/// SplitMix64 hash.
fn mix(i: u64) -> u64 {
    let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Sample `(x, y)` for index `i`: `x` a small signed value with hashed mantissa
/// and exponent in `[2⁻⁶², 1)`, and `y` chosen so `(1+x)ʸ ≈ 2ᴱ` for a target
/// exponent `E` uniform in `[−1070, 1020]` — this keeps the result in range so
/// the near-midpoint filter is not starved, and naturally drives `|y|` huge for
/// the tiny-`x` hard regime.
fn sample(i: u64) -> (f64, f64) {
    let h = mix(i);
    let k = 1 + (h >> 58) % 62; // exponent −k, k ∈ [1, 62]
    let mant = h & 0x000F_FFFF_FFFF_FFFF;
    let mag = f64::from_bits((((1023 - k) as u64) << 52) | mant); // [2⁻ᵏ, 2·2⁻ᵏ)
    let x = if h & (1 << 62) != 0 { -mag } else { mag };

    let h2 = mix(i ^ 0x1234_5678_9ABC_DEF0);
    let e_target = (h2 >> 11) as f64 / (1u64 << 53) as f64 * 2090.0 - 1070.0;
    let y = e_target / (LOG2_E * x.ln_1p());

    (x, y)
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

/// `(1 + x)ʸ` at [`PREC`] bits.
fn oracle(x: f64, y: f64) -> Float {
    (Float::with_val(PREC, x) + 1_u32).pow(Float::with_val(PREC, y))
}

/// `1 + x` is exactly representable as an `f64` — those cases are already covered
/// by the `pow(1+x, y)` cross-check, so the corpus skips them to concentrate on
/// the novel inexact-`1+x` region.
fn one_plus_exact(x: f64) -> bool {
    let s = 1.0 + x;
    let c = if x <= 1.0 {
        x - (s - 1.0)
    } else {
        1.0 - (s - x)
    };
    c == 0.0
}

/// Scan in parallel, returning the surviving `(x, y)` pairs.
fn scan() -> Vec<(f64, f64)> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let chunk = COUNT.div_ceil(threads as u64);
    let done = AtomicU64::new(0);

    let mut found: Vec<(f64, f64)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads as u64)
            .map(|t| {
                let done = &done;
                scope.spawn(move || {
                    let lo = t * chunk;
                    let hi = (lo + chunk).min(COUNT);
                    let mut local = Vec::new();
                    for i in lo..hi {
                        let (x, y) = sample(i);
                        if !(x.is_finite() && y.is_finite() && x > -1.0 && x != 0.0) {
                            continue;
                        }
                        if one_plus_exact(x) {
                            continue;
                        }
                        let v = oracle(x, y);
                        let c = v.to_f64();
                        if c == 0.0 || !c.is_finite() {
                            continue;
                        }
                        if midpoint_frac(&v) < THRESHOLD {
                            local.push((x, y));
                        }
                        if i % (1 << 24) == 0 {
                            let n = done.fetch_add(1 << 24, Ordering::Relaxed) + (1 << 24);
                            eprint!("\r{n} / {COUNT} scanned, {} found ", local.len());
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
    eprintln!("\r{COUNT} scanned, {} found{:20}", found.len(), "");
    found.sort_unstable_by_key(|&(x, y)| (x.to_bits(), y.to_bits()));
    found.dedup_by_key(|&mut (x, y)| (x.to_bits(), y.to_bits()));
    found
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

fn main() {
    let cases = scan();

    let mut out = String::new();
    out.push_str("# Hard-to-round cases for metallic::compound(x, y) = (1 + x)^y.\n");
    out.push_str("# Columns: x, y, correctly-rounded compound(x, y).\n");
    out.push_str(
        "# Generated by `cargo run --release --features mpfr --example gen_f64_compound_cases`.\n",
    );

    let mut mismatches = 0;
    for &(x, y) in &cases {
        let want = oracle(x, y).to_f64();
        let got = metallic::compound(x, y);
        if got.to_bits() != want.to_bits() {
            mismatches += 1;
            eprintln!("MISMATCH compound({x:e}, {y:e}) = {got:e} != {want:e} (correct)");
        }
        writeln!(out, "{}, {}, {}", hexf64(x), hexf64(y), hexf64(want)).unwrap();
    }

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cases/f64_compound.wc");
    std::fs::write(path, out).unwrap();
    eprintln!(
        "wrote {} cases to {path} ({mismatches} current mismatches)",
        cases.len(),
    );
    assert_eq!(mismatches, 0, "metallic::compound is not correctly rounded");
}
