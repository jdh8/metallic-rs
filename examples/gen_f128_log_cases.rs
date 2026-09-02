#![feature(f128)]
//! Generate `tests/cases/log2q.wc`, `tests/cases/log10q.wc` or
//! `tests/cases/log1pq.wc`: the hard-to-round corpus for
//! [`metallic::log2q`], [`metallic::log10q`] or [`metallic::log1pq`], each
//! input with its correctly rounded answer.
//!
//! CORE-MATH has no binary128 `log2`, `log10` or `log1p` yet, so MPFR is the
//! oracle and the answers travel with the inputs; the strict gate then
//! replays under plain `--features f128` with no oracle at all.  Three
//! layers (`log1p` has its own edges and neighbourhoods, listed at
//! [`edges_1p`]):
//!
//! 1. **Edges.** Specials, the neighbours of 1, the fast leg's floor
//!    `|log_b x| = 2^-16`, every power of two (base 2's exact cases,
//!    subnormals included; for base 10 the `j = 0` family, where the sum is
//!    `e·log10 2` alone) and the neighbours of every 32nd one, where the
//!    frame's integer part swamps a 2^-112 fraction; base 10's exact cases
//!    `10^k` for `0 ≤ k ≤ 48` with their neighbours; and significands within
//!    a few ulps of the reduction's table reciprocals `2^(j/2^18)`, where the
//!    reduced `z` is tiny.
//! 2. **The inverse family** (ARITH 2022 § IIB): a hard *output* first — a
//!    114-bit midpoint `z` per result binade and sign — then `x = round(b^z)`,
//!    whose `log_b` lands within `2^-113·log_b e` of that midpoint.
//! 3. **Regression scan.** An MPFR near-midpoint scan, log-uniform over every
//!    binade and bit-uniform over the neighbourhood of 1 where the accurate
//!    leg decides alone, plus a plain random sample of the whole domain.
//!
//! Run with the base (2 or 10), or `1p`, as the argument:
//! ```text
//! CC=clang cargo +nightly run --release --features "f128 mpfr" --example gen_f128_log_cases -- 10
//! ```

use metallic::f128_mpfr::cr_unop;
use rug::Float;
use rug::float::Round;
use rug::ops::Pow;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

/// Working precision of the oracle: how far past the round bit the midpoint
/// distance is resolved.
const PREC: u32 = 250;

/// Keep scan results within this normalized distance of a binary128 midpoint
/// (0 a midpoint, 1 a grid point).
const THRESHOLD: f64 = 1.0 / 65_536.0;

/// Inputs scanned over the whole domain, and over the neighbourhood of 1.
const SCAN: u64 = 100_000_000;
const SCAN_NEAR_ONE: u64 = 50_000_000;

/// Hard outputs per result binade and sign.
const INVERSE: u64 = 32;

/// Plain random inputs kept over the whole domain, no filter.
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

/// A positive value with the exponent field uniform over every binade,
/// subnormals included: the whole domain of the logarithm.
fn positive(i: u64) -> f128 {
    let bits = mix128(i);
    let exponent = (bits >> 112) % 0x7fff;

    f128::from_bits(exponent << 112 | bits & MANTISSA)
}

/// [`positive`] with a sign, negative only above −1: `log1p`'s domain.
fn domain(i: u64) -> f128 {
    let bits = mix128(i);
    let span = if bits & SIGN == 0 { 0x7fff } else { 0x3fff };
    let exponent = (bits >> 112 & 0x7fff) % span;

    f128::from_bits(bits & SIGN | exponent << 112 | bits & MANTISSA)
}

/// `−1 + t` for `t` log-uniform over `[2^-113, 2^-1)`: `1 + x` small and
/// exact, where `log1p`'s reduction cancels.
fn near_minus_one(i: u64) -> f128 {
    let bits = mix128(i);
    let exponent = 16383 - 113 + (bits >> 112 & 0x7fff) % 113;

    -1.0 + f128::from_bits(exponent << 112 | bits & MANTISSA)
}

/// A random sign and significand, the exponent uniform over `[−113, −19]`:
/// the band where `log1p`'s argument is its own reduction.
fn band(i: u64) -> f128 {
    let bits = mix128(i);
    let exponent = 16383 - 113 + (bits >> 112 & 0x7fff) % 95;

    f128::from_bits(bits & SIGN | exponent << 112 | bits & MANTISSA)
}

/// `1 ± t` for `t` log-uniform over `[2^-114, 2^-1)`: the accurate leg's own
/// neighbourhood, where the reduction cancels.
fn near_one(i: u64) -> f128 {
    let bits = mix128(i);
    let exponent = 16383 - 114 + (bits >> 112 & 0x7fff) % 114;
    let t = f128::from_bits(exponent << 112 | bits & MANTISSA);

    if bits & SIGN == 0 { 1.0 + t } else { 1.0 - t }
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

/// `log_b y` in place, to nearest, for `b` 2 or 10 — or `log(1 + y)`.
fn round_log(target: &str, y: &mut Float) -> std::cmp::Ordering {
    match target {
        "2" => y.log2_round(Round::Nearest),
        "10" => y.log10_round(Round::Nearest),
        "1p" => y.ln_1p_round(Round::Nearest),
        _ => unreachable!(),
    }
}

/// The target function of `x`, correctly rounded.
fn logb(target: &str, x: f128) -> f128 {
    cr_unop(x, |y| round_log(target, y))
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

/// `2^k` as binary128, subnormals included.
fn power(k: i32) -> f128 {
    if k < -16382 {
        f128::from_bits(1 << (k + 16494))
    } else {
        f128::from_bits(((k + 16383) as u128) << 112)
    }
}

/// One bit pattern away, `k` times: away from zero for `k > 0`.
fn step(x: f128, k: i128) -> f128 {
    f128::from_bits((x.to_bits() as i128 + k) as u128)
}

/// Layer 1 for `log1p`: specials; the seams, both signs — 2^-113, below
/// which the answer is `x`, 2^-112, where `x²/2` is exactly half an ulp, and
/// 2^-18, where the floating leg hands over to the reduction; `1 + x` a
/// power of two (`x = 2^±k − 1`), where the reduction is exactly zero; every
/// power of two, both signs below 1, with the neighbours of a sparse subset
/// and of every position the 1 can take under the significand up to where it
/// is dropped (2^256); and `1 + x` within a few ulps of a table reciprocal.
fn edges_1p() -> Vec<f128> {
    let mut out = vec![
        0.0,
        -0.0,
        f128::INFINITY,
        f128::NEG_INFINITY,
        f128::NAN,
        -1.0,
        -2.0,
        f128::MAX,
        f128::MIN_POSITIVE,
        -f128::MIN_POSITIVE,
        f128::from_bits(1),
        -f128::from_bits(1),
    ];

    for k in [-113, -112, -18] {
        for x in [power(k), -power(k)] {
            out.extend((-4..=4).map(|d| step(x, d)));
        }
    }
    for k in 1..=113 {
        for x in [power(k) - 1.0, power(-k) - 1.0, (2.0 - power(-k)) - 1.0] {
            out.extend((-2..=2).map(|d| step(x, d)));
        }
    }
    for k in -16494..=16383 {
        let x = power(k);
        out.push(x);
        if k < 0 {
            out.push(-x);
        }
        if k % 32 == 0
            || (111..=115).contains(&k)
            || (253..=257).contains(&k)
            || [-16494, -16493, -16383, -16382, -16381, 16382, 16383].contains(&k)
        {
            out.extend([step(x, -1), step(x, 1)]);
            if k < 0 {
                out.extend([step(-x, -1), step(-x, 1)]);
            }
        }
    }
    for i in 0..512_u64 {
        let bits = mix128(i);
        let j = (bits & 0x3ffff) as u32;
        let e = ((bits >> 64) % 60) as i32 - 17;
        let y: Float = (Float::with_val(PREC, j) / 262_144_u32 + e).exp2();
        let x = (y - 1_u32).to_f128_round(Round::Nearest);
        out.extend((-3..=3).map(|d| step(x, d)));
    }
    out
}

/// Layer 1: the edges.
fn edges(base: u32) -> Vec<f128> {
    let mut out = vec![
        0.0,
        -0.0,
        f128::INFINITY,
        f128::NEG_INFINITY,
        f128::NAN,
        -1.0,
    ];

    // The neighbours of 1, and of the fast leg's floor `|log_b x| = 2^-16`.
    out.extend((-4..=4).map(|d| step(1.0, d)));
    for sign in [1_i32, -1] {
        let t: Float = Float::with_val(PREC, 2).pow(-16_i32) * sign;
        let floor: Float = Float::with_val(PREC, base).pow(t);
        let x = floor.to_f128_round(Round::Nearest);
        out.extend((-2..=2).map(|d| step(x, d)));
    }
    // Every power of two is exact in base 2, and in base 10 leaves the sum
    // `e·log10 2` alone; the neighbours of a sparse subset add a 2^-112
    // fraction under an integer part as large as 2^14.
    for k in -16494..=16383 {
        let x = power(k);
        out.push(x);
        if k % 32 == 0 || [-16494, -16493, -16383, -16382, -16381, 16382, 16383].contains(&k) {
            out.extend([step(x, -1), step(x, 1)]);
        }
    }
    // Base 10's exact cases, `10^k` for `5^k < 2^113`, and their neighbours.
    if base == 10 {
        let mut x = 1.0_f128;
        for _ in 0..=48 {
            out.extend((-2..=2).map(|d| step(x, d)));
            x *= 10.0;
        }
    }
    // Significands a few ulps from a table reciprocal `2^(j/2^18)`, so the
    // reduced `z` is under 2^-110 and the polynomial's squares are nearly
    // zero; `2 − 2^-k` sits within 2^-k of the top one.
    for i in 0..512_u64 {
        let bits = mix128(i);
        let j = (bits & 0x3ffff) as u32;
        let exponent = (bits >> 64) % 0x7ffe + 1;
        let m: Float = Float::with_val(PREC, j) / 262_144_u32;
        let m = m.exp2().to_f128_round(Round::Nearest).to_bits() & MANTISSA;
        let x = f128::from_bits(exponent << 112 | m);
        out.extend((-3..=3).map(|d| step(x, d)));
    }
    for k in 1..=112 {
        let m = (1_u128 << 112) - (1 << (112 - k));
        out.extend([0, 5000, 16382, 32765].map(|e| f128::from_bits(e << 112 | m)));
    }
    out
}

/// Layer 2: per result binade and sign, `x = round(b^z)` for a 114-bit
/// midpoint `z`, so that `log_b x` sits within `2^-113·log_b e` of it — or
/// `x = round(e^z − 1)` for `log1p`, whose result binades reach down to
/// 2^-113.
fn inverse(target: &str) -> Vec<f128> {
    let mut out = Vec::new();
    let floor = if target == "1p" { -113 } else { -20 };

    for exponent in floor..=14_i32 {
        for sign in [1_i32, -1] {
            for i in 0..INVERSE {
                let odd = mix128((exponent as u64) << 40 | i << 8 | u64::from(sign < 0)) >> 15 | 1;
                let z: Float = Float::with_val(PREC, odd)
                    * sign
                    * Float::with_val(PREC, 2).pow(exponent - 113);
                let x = match target.parse::<u32>() {
                    Ok(base) => Float::with_val(PREC, base).pow(z),
                    Err(_) => z.exp_m1(),
                }
                .to_f128_round(Round::Nearest);

                // Base 10 overflows above `z ≈ 4932` and underflows to zero
                // below `z ≈ −4966`: those binades have no input to give.
                if x.is_finite() && x != 0.0 {
                    out.push(x);
                }
            }
        }
    }
    out
}

/// Layer 3: the near-midpoint scan of `sampler`'s first `count` draws, in
/// parallel.
fn scan(target: &str, sampler: fn(u64) -> f128, count: u64, label: &str) -> Vec<f128> {
    let done = AtomicU64::new(0);
    let mut survivors = std::thread::scope(|s| {
        let workers: Vec<_> = (0..THREADS)
            .map(|t| {
                let done = &done;
                s.spawn(move || {
                    let mut kept = Vec::new();
                    for i in (t..count).step_by(THREADS as usize) {
                        let x = sampler(i);
                        let mut y = Float::with_val(PREC, x);
                        round_log(target, &mut y);
                        if midpoint_frac(&y) < THRESHOLD {
                            kept.push(x);
                        }
                        if i % 10_000_000 < THREADS {
                            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                            eprintln!("{label}: {}%", n * 100 / (count / 10_000_000 * THREADS));
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

fn main() {
    let target = std::env::args()
        .nth(1)
        .filter(|s| ["2", "10", "1p"].contains(&s.as_str()))
        .expect("usage: gen_f128_log_cases <2|10|1p>");
    let target = target.as_str();
    let base = target.parse::<u32>().ok();
    let family = inverse(target);
    let (edges, wide, scans) = match base {
        Some(base) => (
            edges(base),
            (0..WIDE).map(positive).collect::<Vec<f128>>(),
            vec![
                (
                    "near a rounding midpoint, from an MPFR scan over every binade",
                    scan(target, positive, SCAN, "domain scan"),
                ),
                (
                    "near a rounding midpoint, from an MPFR scan of 1 ± 2^-114..2^-1",
                    scan(target, near_one, SCAN_NEAR_ONE, "near-1 scan"),
                ),
            ],
        ),
        None => (
            edges_1p(),
            (0..WIDE).map(domain).collect(),
            vec![
                (
                    "near a rounding midpoint, from an MPFR scan over every binade, both signs",
                    scan(target, domain, SCAN, "domain scan"),
                ),
                (
                    "near a rounding midpoint, from an MPFR scan of -1 + 2^-113..2^-1",
                    scan(target, near_minus_one, SCAN_NEAR_ONE, "near -1 scan"),
                ),
                (
                    "near a rounding midpoint, from an MPFR scan of ±2^-113..2^-18",
                    scan(target, band, SCAN_NEAR_ONE, "band scan"),
                ),
            ],
        ),
    };
    eprintln!(
        "{} edges, {} inverse, {:?} near-midpoints",
        edges.len(),
        family.len(),
        scans.iter().map(|(_, s)| s.len()).collect::<Vec<_>>()
    );

    let (edge_title, family_title) = match base {
        Some(10) => (
            "special values, neighbours of 1 and of the fast floor, powers of two and ten",
            "round(10^z) for a 114-bit midpoint z, per result binade and sign",
        ),
        Some(_) => (
            "special values, neighbours of 1 and of the fast floor, powers of two",
            "round(2^z) for a 114-bit midpoint z, per result binade and sign",
        ),
        None => (
            "special values, the seams, 1 + x a power of two, powers of two, near a table reciprocal",
            "round(e^z - 1) for a 114-bit midpoint z, per result binade and sign",
        ),
    };
    let mut text = format!(
        "# Hard-to-round cases for metallic::log{target}q(x), with their correctly rounded answers (MPFR).\n\
         # Generated by `CC=clang cargo +nightly run --release --features \"f128 mpfr\" --example gen_f128_log_cases -- {target}`.\n",
    );
    let mut sections = vec![
        (edge_title, &edges),
        (family_title, &family),
        ("random over the whole domain", &wide),
    ];
    sections.extend(scans.iter().map(|(title, cases)| (*title, cases)));
    for (title, inputs) in sections {
        writeln!(text, "#\n# {title}\n#").unwrap();
        for &x in inputs {
            writeln!(text, "{} {}", hex(x), hex(logb(target, x))).unwrap();
        }
    }
    let path = format!("tests/cases/log{target}q.wc");
    std::fs::write(&path, text).expect("write corpus");
    eprintln!("wrote {path}");
}
