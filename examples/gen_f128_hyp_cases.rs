#![feature(f128)]
//! Frozen MPFR answers for sinhq, coshq, tanhq: mathematical edges, inverse
//! output midpoints, half-ulp cubic corrections, and independent midpoint scans.
//!
//! CC=clang cargo +nightly run --release --features "f128 mpfr" \
//!   --example gen_f128_hyp_cases -- <sinh|cosh|tanh> [samples-per-band]

use metallic::f128_mpfr::cr_unop;
use rug::{Float, float::Round, ops::Pow};
use std::fmt::Write as _;
const PREC: u32 = 512;
const SIGN: u128 = 1 << 127;
const MASK: u128 = (1 << 112) - 1;
fn mix(mut x: u64) -> u64 {
    x = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}
fn bits(i: u64) -> u128 {
    u128::from(mix(i)) | u128::from(mix(i ^ 0x9E37_79B9)) << 64
}
fn power(e: i32) -> Float {
    Float::with_val(PREC, 2).pow(e)
}
fn neighbors(out: &mut Vec<f128>, x: f128, radius: u128) {
    if !x.is_finite() {
        return;
    }
    let b = x.to_bits() & !SIGN;
    for a in b.saturating_sub(radius)..=b + radius {
        out.extend([f128::from_bits(a), f128::from_bits(a | SIGN)]);
    }
}
fn evaluate(kind: usize, y: &mut Float) -> std::cmp::Ordering {
    match kind {
        0 => y.sinh_round(Round::Nearest),
        1 => y.cosh_round(Round::Nearest),
        _ => y.tanh_round(Round::Nearest),
    }
}
fn inverse(kind: usize, z: Float) -> f128 {
    match kind {
        0 => z.asinh(),
        1 => z.acosh(),
        _ => z.atanh(),
    }
    .to_f128_round(Round::Nearest)
}
fn hex(x: f128) -> String {
    let b = x.to_bits();
    let s = if b & SIGN == 0 { "" } else { "-" };
    let e = (b >> 112 & 0x7fff) as i32;
    let m = b & MASK;
    match (e, m) {
        (0x7fff, 0) => format!("{s}inf"),
        (0x7fff, _) => format!("{s}nan"),
        (0, 0) => format!("{s}0x0p+0"),
        (0, _) => format!("{s}0x0.{m:028x}p-16382"),
        _ => format!("{s}0x1.{m:028x}p{:+}", e - 16383),
    }
}
fn edges(kind: usize) -> Vec<f128> {
    let mut out = vec![
        0.0,
        -0.0,
        f128::NAN,
        f128::INFINITY,
        f128::NEG_INFINITY,
        f128::MAX,
        -f128::MAX,
    ];
    // Lindemann-Weierstrass leaves only the zero-input exact cases. All
    // subnormals are in the f(x)=x/1 band, so cover every binary128 exponent.
    for e in -16494..=16383 {
        let x = metallic::ldexpq(1.0, e);
        out.extend([x, -x]);
        if e % 128 == 0 || [-16382, -57, -56, -17, -16, -5, -4, 6, 14].contains(&e) {
            neighbors(&mut out, x, 8);
        }
    }
    for b in 0..1024 {
        out.extend([f128::from_bits(b), f128::from_bits(b | SIGN)]);
    }
    for k in 1..=256 {
        neighbors(&mut out, k as f128, 4);
    }
    if kind < 2 {
        let midpoint = power(16384) - power(16270);
        neighbors(&mut out, inverse(kind, midpoint), 64);
    } else {
        neighbors(
            &mut out,
            inverse(kind, Float::with_val(PREC, 1) - power(-114)),
            64,
        );
    }
    // expq table seams at integer multiples of 2^-18 in log2 space.
    for i in 0..2048 {
        let y = Float::with_val(PREC, (bits(i) & 0x3ffffff) as u32) / 262144_u32;
        let x = y * Float::with_val(PREC, 2).ln();
        neighbors(&mut out, x.to_f128_round(Round::Nearest), 2);
    }
    out
}
fn midpoints(kind: usize) -> Vec<f128> {
    let mut out = Vec::new();
    // Full output range, with dense bins near zero and near tanh's limit.
    let exponents: Vec<i32> = if kind == 2 {
        (-120..=-1).collect()
    } else if kind == 1 {
        (0..=16383).step_by(16).collect()
    } else {
        (-120..=16).chain((17..=16383).step_by(16)).collect()
    };
    for e in exponents {
        for i in 0..32 {
            let odd = (bits((e as u64).wrapping_mul(256).wrapping_add(i)) & ((1 << 114) - 1))
                | 1 << 113
                | 1;
            let z = Float::with_val(PREC, odd) * power(e - 113);
            neighbors(&mut out, inverse(kind, z), 1);
        }
    }
    if kind == 1 || kind == 2 {
        for j in 0..112 {
            for i in 0..32 {
                let odd = (bits(i + j * 32) & ((1 << (j + 1)) - 1)) | 1;
                let delta = Float::with_val(PREC, odd) * power(if kind == 1 { -113 } else { -114 });
                let z = if kind == 1 {
                    Float::with_val(PREC, 1) + delta
                } else {
                    Float::with_val(PREC, 1) - delta
                };
                neighbors(&mut out, inverse(kind, z), 2);
            }
        }
    }
    out
}

/// Solve the small-argument Diophantine family directly: the correction
/// f(x)-x (or x-tanh(x)) equals (j+1/2) ulp(x). The cubic's inverse seeds
/// Newton, then the entire MPFR correction refines it. Rounding x to the
/// 113-bit input lattice places its result near an output midpoint.
fn corrections(kind: usize) -> Vec<f128> {
    if kind == 1 {
        return Vec::new();
    } // cosh's quadratic family is in midpoints.
    let mut out = Vec::new();
    for e in -57..=-5 {
        let ulp = power(e - 112);
        for i in 0..256 {
            let m = (bits((e as u64).wrapping_mul(512).wrapping_add(i)) & MASK) | 1 << 112;
            let x = Float::with_val(PREC, m) * power(e - 112);
            let coefficient: i32 = if kind == 0 { 6 } else { 3 };
            let c = x.clone().pow(3_u32) / coefficient / &ulp;
            let j = c.to_integer_round(Round::Down).unwrap().0;
            let target = (Float::with_val(PREC, j) + 0.5) * &ulp;
            let mut x = Float::with_val(PREC, &target * coefficient).cbrt();
            for _ in 0..8 {
                let (c, derivative) = if kind == 0 {
                    (x.clone().sinh() - &x, x.clone().cosh() - 1)
                } else {
                    let t = x.clone().tanh();
                    (Float::with_val(PREC, &x - &t), t.square())
                };
                x -= (c - &target) / derivative;
            }
            let x = x.to_f128_round(Round::Nearest);
            if x >= power(e).to_f128() && x < power(e + 1).to_f128() {
                neighbors(&mut out, x, 2);
            }
        }
    }
    out
}
fn sample(i: u64, band: u32, kind: usize) -> f128 {
    let b = bits(i);
    let top = if kind == 2 { 5 } else { 13 };
    let e = match band {
        0 => (b >> 112 & 0x7fff) as i32,
        1 => 16383 - 57 + (i % (top + 58)) as i32,
        2 => 16383 - 4 + (i % (top + 5)) as i32,
        _ => 16383 - 57 + (i % 53) as i32,
    };
    f128::from_bits(b & SIGN | (e as u128) << 112 | b & MASK)
}
fn distance(y: Float) -> f64 {
    let rounded = y.to_f128_round(Round::Nearest);
    if !rounded.is_finite() || rounded == 0.0 {
        return 1.0;
    }
    let c = Float::with_val(PREC, rounded);
    let other = if y > c {
        rounded.next_up()
    } else {
        rounded.next_down()
    };
    if !other.is_finite() {
        return 1.0;
    }
    let gap = (Float::with_val(PREC, other) - &c).abs();
    (1.0 - 2.0 * ((y - c).abs() / gap).to_f64()).abs()
}
fn scan(kind: usize, band: u32, count: u64) -> Vec<f128> {
    let mut out = std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..4)
            .map(|t| {
                scope.spawn(move || {
                    let mut kept = Vec::new();
                    for i in (t..count).step_by(4) {
                        let x = sample(i, band, kind);
                        if !x.is_finite() || x.abs() > if kind == 2 { 64.0 } else { 11400.0 } {
                            continue;
                        }
                        let mut y = Float::with_val(PREC, x);
                        evaluate(kind, &mut y);
                        if distance(y) < 1.0 / 65536.0 {
                            kept.push(x);
                        }
                    }
                    kept
                })
            })
            .collect();
        jobs.into_iter()
            .flat_map(|j| j.join().unwrap())
            .collect::<Vec<_>>()
    });
    out.sort_by_key(|x| x.to_bits());
    eprintln!("band {band}: kept {} / {count}", out.len());
    out
}
fn main() {
    let name = std::env::args().nth(1).expect("sinh, cosh or tanh");
    let kind = ["sinh", "cosh", "tanh"]
        .iter()
        .position(|&s| s == name)
        .expect("unknown function");
    let count = std::env::args()
        .nth(2)
        .map_or(5_000_000, |s| s.parse().unwrap());
    let mut sections = vec![
        (
            "exact cases, thresholds, exponent ladder, table seams".to_owned(),
            edges(kind),
        ),
        (
            "inverse 114-bit output midpoints, including the quadratic cosh family".to_owned(),
            midpoints(kind),
        ),
        (
            "half-ulp cubic corrections on the 113-bit input lattice".to_owned(),
            corrections(kind),
        ),
        (
            "deterministic full-representation sample".to_owned(),
            (0..4000).map(|i| sample(i, 0, kind)).collect(),
        ),
    ];
    for band in 0..4 {
        sections.push((
            format!("MPFR midpoint scan, band {band}, {count} samples"),
            scan(kind, band, count),
        ));
    }
    let mut text = format!(
        "# {name}q inputs and correctly rounded MPFR answers.\n# CC=clang cargo +nightly run --release --features \"f128 mpfr\" --example gen_f128_hyp_cases -- {name} {count}\n"
    );
    let mut total = 0;
    for (title, cases) in sections {
        writeln!(text, "#\n# {title}").unwrap();
        total += cases.len();
        for x in cases {
            let y = cr_unop(x, |y| evaluate(kind, y));
            writeln!(text, "{} {}", hex(x), hex(y)).unwrap();
        }
    }
    let path = format!("tests/cases/{name}q.wc");
    std::fs::write(&path, text).unwrap();
    eprintln!("wrote {total} cases to {path}");
}
