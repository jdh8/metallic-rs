#![feature(f128)]
//! Generate `tests/cases/sinq.wc` and `tests/cases/cosq.wc`: the hard-to-round
//! corpora for [`metallic::sinq`] and [`metallic::cosq`], each input with its
//! correctly rounded answer.
//!
//! CORE-MATH has no binary128 sine or cosine yet (only their corpora), so MPFR
//! is the oracle and the answers travel with the inputs; the strict gate then
//! replays under plain `--features f128` with no oracle at all.  Three layers:
//!
//! 1. **Edges.** Specials, the tiny and direct band boundaries, the multiples
//!    of π/2 and the breakpoints `j·π/256` as binary128 rounds them, each with
//!    its neighbours, and the exponents where the 2/π window shifts.
//! 2. **The Diophantine family.** Per binade `e ∈ [0, 16383]`, the significand
//!    `q < 2^113` whose `q·2^(e−112)` lands nearest a multiple `p·π/2`: the
//!    convergents of `2^(e−111)/π`, kept for each parity of `p` — odd puts
//!    the sine near ±1, even the cosine — the same construction as CORE-MATH's
//!    `sin.sage`/`cos.sage`.  These reach within 2^-124 of a multiple of π/2,
//!    which is what the accurate leg's 448-bit residual is sized for.
//! 3. **Regression scan.** An MPFR near-midpoint scan over the reduction and
//!    direct bands, plus a plain random sample of the whole finite range.
//!
//! Run with:
//! ```text
//! CC=clang cargo +nightly run --release --features "f128 mpfr" --example gen_f128_trig_cases
//! ```

use metallic::f128_mpfr::cr_unop;
use rug::float::{Constant, Round};
use rug::{Float, Integer};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

/// Working precision of the oracle in the scan.  MPFR reduces huge arguments
/// exactly on its own; this only sets how far past the round bit the
/// midpoint distance is resolved.
const PREC: u32 = 250;

/// Bits of π for the convergents: past the largest binade plus the 113-bit
/// significand, so the first hundred partial quotients are exact.
const PI_BITS: u32 = 17_000;

/// Keep scan results within this normalized distance of a binary128 midpoint
/// (0 a midpoint, 1 a grid point).
const THRESHOLD: f64 = 1.0 / 65_536.0;

/// Inputs scanned per function over the reduction and direct bands.
const SCAN: u64 = 100_000_000;

/// Plain random inputs kept over the whole finite range, no filter.
const WIDE: u64 = 2_000;

const THREADS: u64 = 16;

const SIGN: u128 = 1 << 127;
const MANTISSA: u128 = (1 << 112) - 1;

fn mix(i: u64) -> u64 {
    let mut z = i.wrapping_mul(0x2545_F491_4F6C_DD1D);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn mix128(i: u64) -> u128 {
    u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9_7F4A_7C15)) << 64
}

/// A signed value with the unbiased exponent drawn from `range`.
fn sample(i: u64, range: core::ops::RangeInclusive<i32>) -> f128 {
    let bits = mix128(i);
    let span = (range.end() - range.start() + 1) as u128;
    let exponent = (*range.start() + 16383) as u128 + (bits >> 112 & 0x7fff) % span;

    f128::from_bits(bits & SIGN | exponent << 112 | bits & MANTISSA)
}

/// CORE-MATH's hexadecimal spelling of a binary128 value.
fn hex(x: f128) -> String {
    let bits = x.to_bits();
    let sign = if bits & SIGN == 0 { "" } else { "-" };
    let exponent = (bits >> 112 & 0x7fff) as i32;
    let mantissa = bits & MANTISSA;

    match (exponent, mantissa) {
        (0x7fff, 0) => format!("{sign}inf"),
        (0x7fff, _) => format!("{sign}nan"),
        (0, 0) => format!("{sign}0x0p+0"),
        (0, _) => format!("{sign}0x0.{mantissa:028x}p-16382"),
        _ => format!("{sign}0x1.{mantissa:028x}p{:+}", exponent - 16383),
    }
}

fn sin(x: f128) -> f128 {
    cr_unop(x, |y| y.sin_round(Round::Nearest))
}

fn cos(x: f128) -> f128 {
    cr_unop(x, |y| y.cos_round(Round::Nearest))
}

/// Normalized distance from `y` to the nearest binary128 midpoint.
///
/// Round-to-nearest is symmetric, so the magnitude decides: stepping the bit
/// pattern of `|c|` up and down is then the neighbouring grid.
fn midpoint_frac(y: &Float) -> f64 {
    let y = &Float::with_val(PREC, y.abs_ref());
    let c = y.to_f128_round(Round::Nearest);
    if !c.is_finite() || c == 0.0 {
        return 1.0;
    }
    let step = |bits: u128| f128::from_bits(bits);
    let up = step(c.to_bits() + 1);
    let down = step(c.to_bits() - 1);
    let mid_hi = (Float::with_val(PREC, c) + Float::with_val(PREC, up)) / 2u32;
    let mid_lo = (Float::with_val(PREC, c) + Float::with_val(PREC, down)) / 2u32;
    let d_hi = Float::with_val(PREC, y - &mid_hi).abs();
    let d_lo = Float::with_val(PREC, y - &mid_lo).abs();
    let c = Float::with_val(PREC, c);

    if d_hi < d_lo {
        (d_hi / (mid_hi - c)).to_f64()
    } else {
        (d_lo / (c - mid_lo)).to_f64()
    }
}

/// Layer 1: the edges, both signs.
fn edges() -> Vec<f128> {
    let pi = Float::with_val(PREC, Constant::Pi);
    let round = |v: Float| v.to_f128_round(Round::Nearest);
    let step = |x: f128, k: i128| f128::from_bits((x.to_bits() as i128 + k) as u128);
    let mut out = vec![
        0.0,
        f128::INFINITY,
        f128::NAN,
        f128::from_bits(1),
        f128::MIN_POSITIVE,
        f128::MAX,
        1.0,
        2.0,
        0.5,
    ];
    // Exponents the bands and the 2/π window turn on.
    for e in [
        -58, -57, -56, -9, -8, -7, 0, 1, 113, 114, 115, 177, 178, 179, 1023, 1024, 16383,
    ] {
        let x = f128::from_bits(((e + 16383) as u128) << 112);
        out.extend([step(x, -1), x, step(x, 1)]);
    }
    // Multiples of π/2 and the breakpoints j·π/256, with neighbours.
    for k in 1..=8 {
        let x = round(Float::with_val(PREC, &pi) * k / 2u32);
        out.extend((-3..=3).map(|d| step(x, d)));
    }
    for j in 1..=512u32 {
        let x = round(Float::with_val(PREC, &pi) * j / 256u32);
        out.extend((-1..=1).map(|d| step(x, d)));
    }
    for k in [3u32, 4, 6] {
        out.push(round(Float::with_val(PREC, &pi) / k));
    }
    let negatives: Vec<f128> = out.iter().map(|&x| -x).collect();
    out.extend(negatives);
    out
}

/// Layer 2: per binade, the significand nearest a multiple of π/2 for each
/// parity of the multiple — the convergents of `2^(e−111)/π`.
fn convergents() -> Vec<f128> {
    let pi = Float::with_val(PI_BITS + 64, Constant::Pi) << PI_BITS;
    let denominator = pi.to_integer().expect("finite");
    let mut out = Vec::new();

    for e in 0..=16383i32 {
        let mut a = Integer::from(1) << (e - 111 + PI_BITS as i32) as u32;
        let mut b = denominator.clone();
        let (mut p0, mut q0) = (Integer::from(0), Integer::from(1));
        let (mut p1, mut q1) = (Integer::from(1), Integer::from(0));
        let mut best: [Option<Integer>; 2] = [None, None];

        loop {
            let (quotient, remainder) = a.div_rem(b.clone());
            let p2 = Integer::from(&quotient * &p1) + &p0;
            let q2 = Integer::from(&quotient * &q1) + &q0;
            if q2.significant_bits() > 113 {
                break;
            }
            best[usize::from(p2.is_odd())] = Some(q2.clone());
            (p0, q0, p1, q1) = (p1, q1, p2, q2);
            if remainder == 0 {
                break;
            }
            a = b;
            b = remainder;
        }
        for q in best.into_iter().flatten() {
            let q = q.to_u128().expect("q < 2^113");
            let shift = q.leading_zeros() - (u128::BITS - 113);
            let exponent = e - shift as i32;
            if exponent < -16382 {
                continue;
            }
            out.push(f128::from_bits(
                ((exponent + 16383) as u128) << 112 | (q << shift) & MANTISSA,
            ));
        }
    }
    out
}

/// Layer 3: the near-midpoint scan over `[2^-57, 2^20)` for `f`, in parallel.
fn scan(f: fn(&mut Float) -> std::cmp::Ordering, label: &str) -> Vec<f128> {
    let done = AtomicU64::new(0);
    let mut survivors = std::thread::scope(|s| {
        let workers: Vec<_> = (0..THREADS)
            .map(|t| {
                let done = &done;
                s.spawn(move || {
                    let mut kept = Vec::new();
                    for i in (t..SCAN).step_by(THREADS as usize) {
                        let x = sample(i, -57..=20);
                        let mut y = Float::with_val(PREC, x);
                        f(&mut y);
                        if midpoint_frac(&y) < THRESHOLD {
                            kept.push(x);
                        }
                        if i % 10_000_000 < THREADS {
                            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                            eprintln!("{label}: {}%", n * 100 / (SCAN / 10_000_000 * THREADS));
                        }
                    }
                    kept
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().expect("worker"))
            .collect::<Vec<_>>()
    });
    survivors.sort_by_key(|x| x.to_bits());
    survivors
}

fn write(path: &str, name: &str, sections: &[(&str, &[f128])], f: fn(f128) -> f128) {
    let mut text = format!(
        "# Hard-to-round cases for metallic::{name}(x), with their correctly rounded answers (MPFR).\n\
         # Generated by `CC=clang cargo +nightly run --release --features \"f128 mpfr\" --example gen_f128_trig_cases`.\n"
    );
    for (title, inputs) in sections {
        writeln!(text, "#\n# {title}\n#").unwrap();
        for &x in *inputs {
            writeln!(text, "{} {}", hex(x), hex(f(x))).unwrap();
        }
    }
    std::fs::write(path, text).expect("write corpus");
    eprintln!("wrote {path}");
}

fn main() {
    let edges = edges();
    let family = convergents();
    eprintln!("{} edge cases, {} convergents", edges.len(), family.len());
    let wide: Vec<f128> = (0..WIDE).map(|i| sample(i, -16382..=16383)).collect();
    let sin_scan = scan(|y| y.sin_round(Round::Nearest), "sin scan");
    let cos_scan = scan(|y| y.cos_round(Round::Nearest), "cos scan");
    eprintln!(
        "{} sin and {} cos near-midpoints",
        sin_scan.len(),
        cos_scan.len()
    );

    write(
        "tests/cases/sinq.wc",
        "sinq",
        &[
            ("special values, band edges, and window edges", &edges),
            (
                "nearest a multiple of pi/2 per binade, both parities",
                &family,
            ),
            ("random over the whole finite range", &wide),
            (
                "near a rounding midpoint, from an MPFR scan over [2^-57, 2^20)",
                &sin_scan,
            ),
        ],
        sin,
    );
    write(
        "tests/cases/cosq.wc",
        "cosq",
        &[
            ("special values, band edges, and window edges", &edges),
            (
                "nearest a multiple of pi/2 per binade, both parities",
                &family,
            ),
            ("random over the whole finite range", &wide),
            (
                "near a rounding midpoint, from an MPFR scan over [2^-57, 2^20)",
                &cos_scan,
            ),
        ],
        cos,
    );
}
