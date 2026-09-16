#![feature(f128)]
//! Freeze atanpiq/atan2piq MPFR answers: exact axes and quarters, table seams,
//! inverse output midpoints, continued fractions of tanPi(midpoint) with
//! 113-bit numerator/denominator, the tiny irrational slope, and MPFR scans.
//!
//! `CC=clang cargo +nightly run --release --features "f128 mpfr"
//! --example gen_f128_atanpi_cases -- 20000000`
#[path = "../tests/all/common/f128_atanpi.rs"]
mod cases;
#[path = "../tests/all/common.rs"]
mod common;
#[path = "../tests/all/common/f128.rs"]
mod common128;

use cases::{MASK as MANTISSA, SIGN};
use rug::{
    Float, Integer,
    float::{Constant, Round},
};
use std::fmt::Write as _;
const PREC: u32 = 600;
const THRESHOLD: f64 = 1.0 / 16384.0;
const THREADS: u64 = 4;

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

/// In the linear band, 2*y/ulp = m*(4/pi or 8/pi) on the normal
/// grid, and 2*n/pi on the subnormal grid. Convergents with odd numerator
/// and their odd multiples approach exactly the rounding midpoints.
fn linear_family() -> Vec<f128> {
    let mut out = Vec::new();
    for numerator in [2u32, 4, 8] {
        let value =
            (Float::with_val(600, numerator) / Float::with_val(600, Constant::Pi)) << 512u32;
        let mut a = value.to_integer().unwrap();
        let mut b = Integer::from(1) << 512u32;
        let (mut p0, mut q0) = (Integer::from(0), Integer::from(1));
        let (mut p1, mut q1) = (Integer::from(1), Integer::from(0));
        loop {
            let (c, rem) = a.div_rem(b.clone());
            let p = Integer::from(&c * &p1) + &p0;
            let q = Integer::from(&c * &q1) + &q0;
            if q.significant_bits() > 113 {
                break;
            }
            if p.is_odd() && q > 0 {
                let q = q.to_u128().unwrap();
                let multiplier = ((1u128 << 112).div_ceil(q)) | 1;
                for m in [q, q * multiplier] {
                    if m >= 1 << 113 {
                        continue;
                    }
                    if numerator == 2 {
                        cases::neighbors(&mut out, f128::from_bits(m), 2);
                    } else {
                        for e in [-16380i32, -1024, -256, -160, -128, -114, -96, -80, -66, -64] {
                            let x = (Float::with_val(113, m) << (e - 112)).to_f128();
                            cases::neighbors(&mut out, x, 2);
                        }
                    }
                }
            }
            (p0, q0, p1, q1) = (p1, q1, p, q);
            if rem == 0 {
                break;
            }
            a = b;
            b = rem;
        }
    }
    out
}

/// Convergents of a positive real whose two integers fit exactly in binary128.
fn convergents(value: &Float) -> Vec<(u128, u128)> {
    let mut a = Float::with_val(
        PREC,
        value * Float::with_val(PREC, Integer::from(1) << 512u32),
    )
    .to_integer_round(Round::Down)
    .unwrap()
    .0;
    let mut b = Integer::from(1) << 512u32;
    let (mut p0, mut q0) = (Integer::from(0), Integer::from(1));
    let (mut p1, mut q1) = (Integer::from(1), Integer::from(0));
    let mut out = Vec::new();
    loop {
        let (c, rem) = a.div_rem(b.clone());
        let p = Integer::from(&c * &p1) + &p0;
        let q = Integer::from(&c * &q1) + &q0;
        if p.significant_bits() > 113 || q.significant_bits() > 113 {
            break;
        }
        if p > 0 && q > 0 && p.significant_bits().max(q.significant_bits()) >= 100 {
            out.push((p.to_u128().unwrap(), q.to_u128().unwrap()));
        }
        (p0, q0, p1, q1) = (p1, q1, p, q);
        if rem == 0 {
            break;
        }
        a = b;
        b = rem;
    }
    out
}

/// Map a 114-bit output midpoint back through tanPi. For atan2, first fold
/// into [0,1/4] and normalize the tangent so its continued fractions resolve
/// 113-bit significands even when the ratio's exponent is near underflow.
fn inverse_one(unary: &mut Vec<f128>, binary: &mut Vec<[f128; 2]>, z: Float) {
    if z < 0.5 {
        cases::neighbors(unary, z.clone().tan_pi().to_f128(), 1);
    }
    let (a, swap, xneg) = if z <= 0.25 {
        (z, false, false)
    } else if z <= 0.5 {
        (Float::with_val(PREC, 0.5) - z, true, false)
    } else if z <= 0.75 {
        (z - 0.5, true, true)
    } else {
        (Float::with_val(PREC, 1) - z, false, true)
    };
    let t = a.tan_pi();
    if t == 0 {
        return;
    }
    let e = t.get_exp().unwrap() - 1;
    let normalized = t >> e;
    // The last fitting convergent is the closest member of this family.
    let Some((p, q)) = convergents(&normalized).pop() else {
        return;
    };
    // A common scale leaves the ratio exact; 256 keeps both operands normal
    // even when the target output midpoint is subnormal.
    let shift = if e < -160 { 256 } else { 0 };
    let y = metallic::ldexpq(p as f128, e + shift);
    let x = metallic::ldexpq(q as f128, shift);
    let (y, x) = if swap { (x, y) } else { (y, x) };
    let x = if xneg { -x } else { x };
    binary.extend([[y, x], [-y, x]]);
    if shift == 0 {
        for near in [y.next_down(), y.next_up()] {
            binary.extend([[near, x], [-near, x]]);
        }
    }
}

fn inverse() -> (Vec<f128>, Vec<[f128; 2]>) {
    let (mut unary, mut binary) = (Vec::new(), Vec::new());
    for e in -16494..=-1 {
        for k in 0..if e < -160 { 1 } else { 32 } {
            let h = common128::mix128(((e + 16494) * 32 + k) as u64);
            let m = (1 << 113) | ((h & MANTISSA) << 1) | 1;
            inverse_one(
                &mut unary,
                &mut binary,
                Float::with_val(PREC, m) << (e - 113),
            );
        }
    }
    for k in 0..2048u32 {
        // The spacings differ on the two sides of powers of two. Form
        // each midpoint on its own grid, also around 3/4 and just below 1.
        for center in [0.25_f128, 0.5, 0.75, 1.0] {
            let lo = Float::with_val(PREC, center) - Float::with_val(PREC, center.next_down());
            let d = lo * (2 * k + 1) / 2u32;
            inverse_one(&mut unary, &mut binary, Float::with_val(PREC, center) - d);
            if center < 1.0 {
                let hi = Float::with_val(PREC, center.next_up()) - Float::with_val(PREC, center);
                let d = hi * (2 * k + 1) / 2u32;
                inverse_one(&mut unary, &mut binary, Float::with_val(PREC, center) + d);
            }
        }
        inverse_one(
            &mut unary,
            &mut binary,
            Float::with_val(PREC, 2 * k + 1) >> 16495,
        );
    }
    (unary, binary)
}

/// Independent 300-bit MPFR scans keep near-midpoints on each IEEE grid.
fn scan(count: u64) -> (Vec<f128>, Vec<[f128; 2]>) {
    let mut kept = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..THREADS)
            .map(|thread| {
                scope.spawn(move || {
                    let (mut unary, mut binary) = (Vec::new(), Vec::new());
                    for block in (thread..count.div_ceil(256)).step_by(THREADS as usize) {
                        for i in block * 256..((block + 1) * 256).min(count) {
                            let x = match i % 4 {
                                0 => cases::banded(i),
                                1 => cases::seams(i),
                                2 => cases::domain(i),
                                _ => cases::subnormal(i),
                            };
                            if midpoint_frac(&Float::with_val(300, x).atan_pi()) < THRESHOLD {
                                unary.push(x);
                            }
                            let [y, x] = match i % 4 {
                                0 => cases::banded_pairs(i),
                                1 => cases::seam_pairs(i),
                                2 => cases::pairs(i),
                                _ => cases::tiny_pairs(i),
                            };
                            if midpoint_frac(
                                &Float::with_val(300, y).atan2_pi(&Float::with_val(300, x)),
                            ) < THRESHOLD
                            {
                                binary.push([y, x]);
                            }
                            if i % 1_000_000 == 0 {
                                eprintln!("scan: {i}/{count}");
                            }
                        }
                    }
                    (unary, binary)
                })
            })
            .collect();
        workers.into_iter().map(|w| w.join().unwrap()).fold(
            (Vec::new(), Vec::new()),
            |(mut a, mut b), (u, v)| {
                a.extend(u);
                b.extend(v);
                (a, b)
            },
        )
    });
    kept.0.sort_by_key(|x| x.to_bits());
    kept.1.sort_by_key(|[y, x]| (y.to_bits(), x.to_bits()));
    kept
}

fn main() {
    let count = std::env::args()
        .nth(1)
        .map_or(20_000_000, |s| s.parse().expect("scan count"));
    let (inverse, inverse2) = inverse();
    eprintln!(
        "inverse family: {} unary, {} binary inputs",
        inverse.len(),
        inverse2.len()
    );
    let (scan, scan2) = scan(count);
    let linear = linear_family();
    let edges = cases::edges();
    // MPFR sweeps test the full per-binade exact family; freeze a sparse
    // selection here together with all axes, subnormals and table seams.
    let edges2: Vec<_> = cases::edge_pairs()
        .into_iter()
        .enumerate()
        .filter_map(|(i, pair)| (i < 100_000 || i % 64 == 0).then_some(pair))
        .collect();
    let wide: Vec<_> = (0..65536).map(cases::domain).collect();
    let wide2: Vec<_> = (0..65536).map(cases::pairs).collect();
    let linear2: Vec<_> = linear
        .iter()
        .flat_map(|&y| [[y, 1.0], [y, -1.0], [1.0, y], [-1.0, y]])
        .collect();
    for (name, binary) in [("atanpiq", false), ("atan2piq", true)] {
        let mut text = format!(
            "# {name}: precision-113 MPFR answers with ternary-aware IEEE subnormalization.\n\
             # Generated by `CC=clang cargo +nightly run --release --features \"f128 mpfr\" --example gen_f128_atanpi_cases -- {count}`.\n"
        );
        let mut seen = std::collections::HashSet::new();
        let mut mismatches = 0;
        for (title, unary, pairs) in [
            (
                "exact axes/quarters, grid edges and reduction seams",
                &edges,
                &edges2,
            ),
            (
                "continued fractions of 2/pi, 4/pi and 8/pi on the tiny grids",
                &linear,
                &linear2,
            ),
            (
                "inverse output midpoints and 113-bit convergents of tanPi(midpoint)",
                &inverse,
                &inverse2,
            ),
            ("full-representation regression sample", &wide, &wide2),
            ("MPFR near-midpoint scan", &scan, &scan2),
        ] {
            writeln!(text, "#\n# {title}\n#").unwrap();
            let inputs: Box<dyn Iterator<Item = [f128; 2]>> = if binary {
                Box::new(pairs.iter().copied())
            } else {
                Box::new(unary.iter().map(|&x| [x, 1.0]))
            };
            for [y, x] in inputs {
                if !seen.insert((y.to_bits(), x.to_bits())) {
                    continue;
                }
                let (got, want) = if binary {
                    (metallic::atan2piq(y, x), cases::oracle2(y, x))
                } else {
                    (metallic::atanpiq(y), cases::oracle(y))
                };
                if got.to_bits() != want.to_bits() && !(got.is_nan() && want.is_nan()) {
                    if mismatches < 20 {
                        eprintln!("{name}({y:?},{x:?}) = {got:?}, want {want:?}");
                    }
                    mismatches += 1;
                }
                if binary {
                    writeln!(text, "{} {} {}", hex(y), hex(x), hex(want)).unwrap();
                } else {
                    writeln!(text, "{} {}", hex(y), hex(want)).unwrap();
                }
            }
        }
        assert_eq!(mismatches, 0, "{name} corpus mismatches");
        let path = format!("tests/cases/{name}.wc");
        std::fs::write(&path, text).unwrap();
        eprintln!("wrote {path}: {} cases, 0 mismatches", seen.len());
    }
}
