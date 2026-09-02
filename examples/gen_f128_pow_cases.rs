#![feature(f128)]
//! Generate `tests/cases/powq.wc`: the hard-to-round corpus for
//! [`metallic::powq`], each input pair with its correctly rounded answer.
//!
//! CORE-MATH has no binary128 `pow` yet, so MPFR is the oracle and the answers
//! travel with the inputs; the strict gate then replays under plain
//! `--features f128` with no oracle at all.  Three layers:
//!
//! 1. **Edges.** The special values of Annex F; powers of two under dyadic
//!    exponents, exact down to the subnormal midpoint `2^-16495` and up to the
//!    overflow boundary; the exact and midpoint family — `M^y` for integer `y`
//!    and `m^N` for `x = m^(2^k)·2^E`, `y = N/2^k` — at 113 and 114 bits with
//!    their neighbours; the thresholds where the result rounds to 1, to ∞, to
//!    the least subnormal and to 0; and the fast leg's hand-over at
//!    `|y| = 2^25`.
//! 2. **The inverse family** (ARITH 2022 § IIB): a hard *output* first — a
//!    114-bit midpoint `z` — under a small dyadic `y`, so that
//!    `x = round(z^(1/y))` perturbs `x^y` by only `|y|·2^-113` relative and
//!    leaves it within that of the midpoint.
//! 3. **Regression scan.** An MPFR near-midpoint scan over the finite domain,
//!    the neighbourhood of 1 under large exponents, integer exponents on
//!    bases of either sign, and the benchmark's band, plus a plain random
//!    sample of the whole domain.
//!
//! ```text
//! CC=clang cargo +nightly run --release --features "f128 mpfr" --example gen_f128_pow_cases
//! ```

use metallic::f128_mpfr::cr_binop;
use rug::Float;
use rug::float::Round;
use rug::ops::{Pow, PowAssignRound};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

/// Working precision of the oracle: how far past the round bit the midpoint
/// distance is resolved.
const PREC: u32 = 250;

/// Keep scan results within this normalized distance of a binary128 midpoint
/// (0 a midpoint, 1 a grid point).
const THRESHOLD: f64 = 1.0 / 65_536.0;

/// Pairs scanned per distribution.
const SCAN: u64 = 50_000_000;

/// Hard outputs per result binade.
const INVERSE: u64 = 24;

/// Plain random pairs kept over the whole domain, no filter.
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

/// A positive `x` with the exponent uniform over every binade, subnormals
/// included, under a `y` of either sign whose exponent keeps `|y·log2 x|`
/// under 2^14 most of the time.
fn general(i: u64) -> [f128; 2] {
    let xb = mix128(2 * i);
    let yb = mix128(2 * i + 1);
    let ex = (xb >> 112) % 0x7fff;
    let scale = (ex as i64 - 16383).unsigned_abs().max(1).ilog2() as i64;
    let ey = 16383 - 120 + ((yb >> 112) % (135 - scale) as u128) as i64;

    [
        f128::from_bits(ex << 112 | xb & MANTISSA),
        f128::from_bits(yb & SIGN | (ey as u128) << 112 | yb & MANTISSA),
    ]
}

/// `x = 1 ± t` for `t` log-uniform over `[2^-113, 2^-1)` under a `y` large
/// enough for `|y·log2 x|` to reach 2^13.
fn near_one(i: u64) -> [f128; 2] {
    let xb = mix128(2 * i);
    let yb = mix128(2 * i + 1);
    let k = (xb >> 112 & 0x7fff) % 113;
    let t = f128::from_bits((16383 - 113 + k) << 112 | xb & MANTISSA);
    let ey = 16383 + k as i64 - 120 + ((yb >> 112) % 134) as i64;

    [
        if xb & SIGN == 0 { 1.0 + t } else { 1.0 - t },
        f128::from_bits(yb & SIGN | (ey as u128) << 112 | yb & MANTISSA),
    ]
}

/// Integer exponents in `[−300, 300]` on a base of either sign in `[1, 4)`.
fn integers(i: u64) -> [f128; 2] {
    let xb = mix128(2 * i);

    [
        f128::from_bits(xb & SIGN | (16383 + (xb >> 120 & 1)) << 112 | xb & MANTISSA),
        (mix(i) % 601) as f128 - 300.0,
    ]
}

/// The benchmark's band: `|x| ∈ [2^-16, 2^17)`, `|y| ∈ [2^-16, 2^9)`.
fn band(i: u64) -> [f128; 2] {
    let xb = mix128(2 * i);
    let yb = mix128(2 * i + 1);
    let ex = 16383 - 16 + (xb >> 112) % 33;
    let ey = 16383 - 16 + (yb >> 112) % 25;

    [
        f128::from_bits(ex << 112 | xb & MANTISSA),
        f128::from_bits(yb & SIGN | ey << 112 | yb & MANTISSA),
    ]
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

/// `x^y`, correctly rounded.
fn pow(x: f128, y: f128) -> f128 {
    cr_binop(x, y, |a, b| a.pow_assign_round(b, Round::Nearest))
}

/// Normalized distance from `y` to the nearest binary128 midpoint.
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

/// The odd `N` whose `m^N` has 113 or 114 bits, if any (at most one each).
fn tall_powers(m: u128) -> Vec<(u32, u128)> {
    let mut out = Vec::new();
    let mut value = m;
    let mut n = 1;

    while value < 1 << 114 {
        if value >= 1 << 113 {
            out.push((n, value));
        }
        match value.checked_mul(m).and_then(|v| v.checked_mul(m)) {
            Some(next) => value = next,
            None => break,
        }
        n += 2;
    }
    out
}

/// Layer 1: the edges.
fn edges() -> Vec<[f128; 2]> {
    let inf = f128::INFINITY;
    let nan = f128::NAN;
    let mut out = vec![
        [nan, 0.0],
        [nan, -0.0],
        [1.0, nan],
        [1.0, inf],
        [-1.0, inf],
        [-1.0, -inf],
        [-1.0, nan],
        [nan, 1.0],
        [2.0, nan],
        [nan, nan],
        [-2.0, 0.5],
        [-2.0, 1.5],
        [-f128::MIN_POSITIVE, 0.5],
        [-inf, 0.5],
        [-inf, 3.0],
        [-inf, -3.0],
        [-inf, -2.0],
        [-inf, 2.0],
        [inf, -0.5],
        [inf, 0.5],
        [inf, inf],
        [inf, -inf],
        [0.5, inf],
        [0.5, -inf],
        [1.5, inf],
        [1.5, -inf],
        [0.0, inf],
        [0.0, -inf],
        [-0.0, inf],
        [-0.0, -inf],
        [0.0, 3.0],
        [-0.0, 3.0],
        [-0.0, 2.0],
        [-0.0, 0.5],
        [0.0, -3.0],
        [-0.0, -3.0],
        [-0.0, -0.5],
        [-0.0, -2.0],
        [-2.0, 3.0],
        [-2.0, 4.0],
        [-2.0, -2.0],
        [-2.0, -3.0],
        [-1.0, 3.0],
        [-1.0, 1e30],
        [-1.0, f128::MAX],
        [-1.0, -f128::MAX],
        [-1.0, f128::MIN_POSITIVE],
        [f128::MAX, 2.0],
        [f128::MAX, -2.0],
        [f128::MAX, 1.0],
        [f128::MAX, -1.0],
        [f128::MIN_POSITIVE, 2.0],
        [f128::MIN_POSITIVE, -2.0],
        [f128::from_bits(1), 0.5],
        [f128::from_bits(1), 1.0],
        [f128::from_bits(1), -1.0],
        [f128::from_bits(1), f128::from_bits(1)],
        [f128::from_bits(MANTISSA), 1.0],
        [2.0, 0.5],
        [3.0, 1.0 / 3.0],
        [10.0, 0.5],
        [1.0 + f128::EPSILON, 1e30],
        [1.0 - f128::EPSILON, 1e30],
    ];
    let one_ulp = [1.0 + f128::EPSILON, 1.0 - f128::EPSILON / 2.0];

    // Powers of two under dyadic exponents: `2^G` exactly, with `G` sweeping
    // the overflow boundary, the least normal, the least subnormal, and the
    // subnormal midpoint that rounds to zero.
    for k in [1, -1, 2, -2, 3, 5, -5, 7, 64, -64, 16382, -16382, -16494] {
        let x = power(k);
        for g in [
            16383_i64, 16384, 16385, -16382, -16383, -16493, -16494, -16495, -16496, 0, 1, -1, 113,
            -113, 114, -114,
        ] {
            let y = g as f128 / k as f128;
            if (y * k as f128) as i64 == g {
                out.push([x, y]);
                out.push([x, step(y, 1)]);
                out.push([x, step(y, -1)]);
                out.push([step(x, 1), y]);
                out.push([step(x, -1), y]);
            }
        }
        for n in 1..=8 {
            out.push([x, n as f128 / 8.0]);
            out.push([x, -(n as f128) / 8.0]);
            out.push([x, (n as f128 + 1.0 / 3.0) as f128]);
        }
    }
    // The exact and midpoint family, and the neighbours of its members.
    let mut family = Vec::new();
    for y in 2..=71_u32 {
        // `M^y` with 113 or 114 bits: `M` around `2^(113/y)` and `2^(114/y)`.
        for anchor in [113.0, 114.0] {
            let center = 2_f64.powf(anchor / f64::from(y)).round() as u128 | 1;
            let mut m = center.saturating_sub(8) | 1;
            while m <= center + 8 {
                let width = m.checked_pow(y).map_or(200, |v| 128 - v.leading_zeros());
                if (112..=114).contains(&width) {
                    family.push((m, f128::from(y)));
                }
                m += 2;
            }
        }
        for m in [3_u128, 5, 7, 9, 11, 13, 15] {
            if m.checked_pow(y).is_some_and(|v| v < 1 << 114) {
                family.push((m, f128::from(y)));
            }
        }
    }
    for k in 1..=6_u32 {
        // Up to a hundred perfect 2^k-th powers per level: every odd root
        // where there are few, a hashed stride through them otherwise.
        let limit = 2_f64.powf(113.0 / f64::from(1_u32 << k)).floor() as u128;
        let stride = (limit / 100).max(1) * 2;
        let mut m = 3_u128;
        while m <= limit {
            let base = m.pow(1 << k);
            let scale = f64::from(1_u32 << k) as f128;
            family.push((base, 1.0 / scale));
            for (n, _) in tall_powers(m) {
                family.push((base, n as f128 / scale));
            }
            m += if stride == 2 {
                2
            } else {
                u128::from(mix(m as u64) % stride as u64 & !1) + 2
            };
        }
    }
    for (i, &(m, y)) in family.iter().enumerate() {
        let x = m as f128;
        // Binade shifts that keep `E·y` an integer — multiples of 64 cover
        // every `2^k` in play — one of them hashed far out, where the result
        // reaches the subnormal or overflow boundary.
        let e = (mix(i as u64) % 512) as i32 * 64 - 16384;
        for e in [
            0,
            64 * (1 + (i % 7) as i32) * if i % 2 == 0 { 1 } else { -1 },
            e,
        ] {
            let xe = x * power(e);
            if xe.is_finite() && xe != 0.0 {
                out.push([xe, y]);
                out.push([-xe, y]);
                out.push([step(xe, 1), y]);
                out.push([step(xe, -1), y]);
                out.push([xe, step(y, 1)]);
                out.push([xe, step(y, -1)]);
            }
        }
    }
    // Thresholds: `y = g/log2 x` for the results 1 ± 2^-114, 1 ± 2^-113,
    // 2^16384, the least normal, the least subnormal and its midpoint.
    for i in 0..256_u64 {
        let bits = mix128(i);
        let e = (bits >> 112 & 0x7fff) % 0x7ffe + 1;
        let x = f128::from_bits(e << 112 | bits & MANTISSA);
        let l = Float::with_val(PREC, x).log2();
        for g in [
            Float::with_val(PREC, 2).pow(-114_i32),
            Float::with_val(PREC, 2).pow(-113_i32),
            -Float::with_val(PREC, 2).pow(-114_i32),
            -Float::with_val(PREC, 2).pow(-115_i32),
            Float::with_val(PREC, 16384),
            Float::with_val(PREC, -16382),
            Float::with_val(PREC, -16494),
            Float::with_val(PREC, -16495),
            Float::with_val(PREC, -16496),
        ] {
            let y = Float::with_val(PREC, &g / &l).to_f128_round(Round::Nearest);
            if y.is_finite() && y != 0.0 {
                out.extend((-2..=2).map(|d| [x, step(y, d)]));
            }
        }
    }
    // The fast leg's hand-over at `|y| = 2^25`, on bases near 1 so the result
    // stays finite, and the accurate leg's floating logarithm under huge `y`.
    for i in 0..256_u64 {
        let bits = mix128(i);
        let k = 16383 - 20 - (bits >> 112 & 0x7fff) % 93;
        let t = f128::from_bits(k << 112 | bits & MANTISSA);
        let x = if bits & SIGN == 0 { 1.0 + t } else { 1.0 - t };
        let y = power(25);
        out.extend((-2..=2).map(|d| [x, step(y, d)]));
        out.extend((-2..=2).map(|d| [x, -step(y, d)]));
        let big = power(110 - (16383 - k as i32));
        out.push([x, big]);
        out.push([x, -big]);
    }
    for x in one_ulp {
        for k in 0..=125 {
            out.push([x, power(k)]);
            out.push([x, -power(k)]);
        }
    }
    out
}

/// Layer 2: per result binade near 1, `x = round(z^(1/y))` for a 114-bit
/// midpoint `z` and a small dyadic `y`, so that `x^y` sits within
/// `|y|·2^-113` of the midpoint.
fn inverse() -> Vec<[f128; 2]> {
    let mut out = Vec::new();

    for exponent in -16..=16_i32 {
        for i in 0..INVERSE {
            let bits = mix128((exponent as u64) << 40 | i << 8);
            let odd = bits >> 15 | 1;
            let z: Float =
                Float::with_val(PREC, odd) * Float::with_val(PREC, 2).pow(exponent - 113);
            let k = 8 + (bits >> 100) % 48;
            let n = (bits >> 40 & 0xffff) as u32 | 1;
            let y =
                f128::from(n) / (1_u128 << k) as f128 * if bits & SIGN == 0 { 1.0 } else { -1.0 };
            let x = z
                .pow(Float::with_val(PREC, 1) / Float::with_val(PREC, y))
                .to_f128_round(Round::Nearest);
            if x.is_finite() && x != 0.0 {
                out.push([x, y]);
            }
        }
    }
    out
}

/// Layer 3: the near-midpoint scan of `sampler`'s first `count` draws, in
/// parallel.
fn scan(sampler: fn(u64) -> [f128; 2], count: u64, label: &str) -> Vec<[f128; 2]> {
    let done = AtomicU64::new(0);
    let mut survivors = std::thread::scope(|s| {
        let workers: Vec<_> = (0..THREADS)
            .map(|t| {
                let done = &done;
                s.spawn(move || {
                    let mut kept = Vec::new();
                    for i in (t..count).step_by(THREADS as usize) {
                        let [x, y] = sampler(i);
                        let mut r = Float::with_val(PREC, x);
                        r.pow_assign_round(&Float::with_val(PREC, y), Round::Nearest);
                        if midpoint_frac(&r) < THRESHOLD {
                            kept.push([x, y]);
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
    survivors.sort_by_key(|[x, y]| (x.to_bits(), y.to_bits()));
    survivors
}

fn main() {
    let edges = edges();
    let family = inverse();
    let wide: Vec<[f128; 2]> = (0..WIDE).map(general).collect();
    let scans = [
        (
            "near a rounding midpoint, from an MPFR scan over the finite domain",
            scan(general, SCAN, "domain scan"),
        ),
        (
            "near a rounding midpoint, from an MPFR scan of 1 ± 2^-113..2^-1 under large exponents",
            scan(near_one, SCAN / 2, "near-1 scan"),
        ),
        (
            "near a rounding midpoint, from an MPFR scan of integer exponents on bases of either sign",
            scan(integers, SCAN / 2, "integer scan"),
        ),
        (
            "near a rounding midpoint, from an MPFR scan of |x| in [2^-16, 2^17), |y| in [2^-16, 2^9)",
            scan(band, SCAN, "band scan"),
        ),
    ];
    eprintln!(
        "{} edges, {} inverse, {:?} near-midpoints",
        edges.len(),
        family.len(),
        scans.iter().map(|(_, s)| s.len()).collect::<Vec<_>>()
    );

    let mut text = String::from(
        "# Hard-to-round cases for metallic::powq(x, y), with their correctly rounded answers (MPFR).\n\
         # Generated by `CC=clang cargo +nightly run --release --features \"f128 mpfr\" --example gen_f128_pow_cases`.\n",
    );
    let mut sections = vec![
        (
            "special values, powers of two, the exact and midpoint family, the thresholds, the fast leg's hand-over",
            &edges,
        ),
        (
            "round(z^(1/y)) for a 114-bit midpoint z and a small dyadic y",
            &family,
        ),
        ("random over the whole domain", &wide),
    ];
    sections.extend(scans.iter().map(|(title, cases)| (*title, cases)));
    for (title, inputs) in sections {
        writeln!(text, "#\n# {title}\n#").unwrap();
        for &[x, y] in inputs {
            writeln!(text, "{} {} {}", hex(x), hex(y), hex(pow(x, y))).unwrap();
        }
    }
    let path = "tests/cases/powq.wc";
    std::fs::write(path, text).expect("write corpus");
    eprintln!("wrote {path}");
}
