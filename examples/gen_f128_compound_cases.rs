#![feature(f128)]
//! Generate the answer-carrying compoundq corpus: exact and midpoint powers,
//! wide perfect roots of 1+x, small-rate half-ulp products, inverse output
//! midpoints, and four deterministic near-midpoint scans (20M pairs total).
//!
//! CC=clang cargo +nightly run --release --features "f128 mpfr" \
//!   --example gen_f128_compound_cases -- [samples-per-band=5000000]
use metallic::f128_mpfr::cr_compound;
use rug::{Float, float::Round, ops::Pow};
use std::fmt::Write as _;
const PREC: u32 = 384;
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
fn power(e: i32) -> f128 {
    metallic::ldexpq(1.0, e)
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
fn neighbors(out: &mut Vec<[f128; 2]>, x: f128, y: f128, radius: i128) {
    if !(x.is_finite() && y.is_finite()) {
        return;
    }
    for d in -radius..=radius {
        out.push([f128::from_bits(x.to_bits().wrapping_add_signed(d)), y]);
        out.push([x, f128::from_bits(y.to_bits().wrapping_add_signed(d))]);
    }
}
fn edges() -> Vec<[f128; 2]> {
    let mut out = Vec::new();
    let special = [
        0.0,
        -0.0,
        -1.0,
        1.0,
        -2.0,
        0.5,
        -0.5,
        f128::INFINITY,
        f128::NEG_INFINITY,
        f128::NAN,
        f128::MAX,
        f128::MIN_POSITIVE,
        f128::from_bits(1),
    ];
    for x in special {
        for y in special {
            out.push([x, y]);
        }
    }
    for e in -113..=113 {
        let x = power(e) - 1.0;
        if x <= -1.0 || x == 0.0 {
            continue;
        }
        for y in [
            -16495.0, -16494.0, -16382.0, -114.0, -1.0, 0.5, 1.0, 2.0, 16383.0, 16384.0,
        ] {
            neighbors(&mut out, x, y / e as f128, 2);
        }
    }
    // Odd integer powers reaching 113/114 bits, and their dyadic roots.
    for m in (3..100_u128).step_by(2) {
        for n in 1..=71 {
            let Some(v) = m.checked_pow(n) else {
                break;
            };
            if v >= 1 << 114 {
                break;
            }
            if v < 1 << 112 {
                continue;
            }
            for k in 0..=6 {
                let Some(base) = m.checked_pow(1 << k) else {
                    break;
                };
                if base >= 1 << 113 {
                    break;
                }
                neighbors(
                    &mut out,
                    base as f128 - 1.0,
                    n as f128 / (1_u128 << k) as f128,
                    2,
                );
            }
        }
    }
    // Perfect squares with bases up to 228 bits: (2^k +/- 1)^2 - 1.
    for k in 2..=114 {
        for sign in [-1.0, 1.0] {
            let x = power(2 * k) + sign * power(k + 1);
            neighbors(&mut out, x, 0.5, 2);
        }
    }
    // Near-one result midpoints: exact x*y = +/-2^-113 or +/-2^-114.
    // Scan every input binade, including the least subnormal; the binomial
    // correction determines which side of that midpoint the answer occupies.
    for e in -16494..=-1 {
        for sx in [-1.0, 1.0] {
            for sy in [-1.0, 1.0] {
                let target = if sx * sy > 0.0 { -113 } else { -114 };
                let y = sy * power(target - e);
                if y.is_finite() {
                    let x = sx * power(e);
                    out.push([x, y]);
                    if e % 128 == 0 || (-120..=-108).contains(&e) {
                        neighbors(&mut out, x, y, 2);
                    }
                }
            }
        }
    }
    for e in [
        -16494, -16382, -1000, -640, -256, -114, -113, -112, -19, -18, -17, -1, 0, 1, 112, 113,
        114, 225, 226, 255, 256, 639, 640, 16383,
    ] {
        for sign in [-1.0, 1.0] {
            let x = sign * power(e);
            if x <= -1.0 {
                continue;
            }
            for y in [
                -power(25),
                -1.0,
                0.5,
                1.0,
                power(25),
                power((-e).clamp(-16382, 16383)),
            ] {
                neighbors(&mut out, x, y, 3);
            }
        }
    }
    out
}

// Inverse construction uses expm1 so rates far below one ulp of 1 survive.
// Fix both signs of a dyadic exponent; start from exact 114-bit midpoints,
// including the subnormal and overflow rounding boundaries.
fn inverse() -> Vec<[f128; 2]> {
    let mut out = Vec::new();
    for ez in [-16495, -16494, -16383, -100, -1, 0, 1, 100, 16383] {
        for i in 0..64 {
            let a = bits(i);
            let z: Float = Float::with_val(PREC, (1_u128 << 113) | (a & ((1 << 113) - 1)) | 1)
                * Float::with_val(PREC, 2).pow(ez - 113);
            for ey in [-10, -1, 0, 1, 25, 100, 1000, 16000] {
                for sign in [-1.0, 1.0] {
                    let y = sign * power(ey);
                    let x = (z.clone().ln() / y).exp_m1().to_f128_round(Round::Nearest);
                    if x > -1.0 && x.is_finite() {
                        neighbors(&mut out, x, y, 1);
                    }
                }
            }
        }
    }
    out
}
fn sample(kind: usize, i: u64) -> [f128; 2] {
    let a = bits(2 * i);
    let b = bits(2 * i + 1);
    let e = match kind {
        0 => ((a >> 112) % 32767) as i32 - 16383,
        1 => -120 + ((a >> 112) % 120) as i32,
        2 => -18 + ((a >> 112) % 32) as i32,
        _ => -16383,
    };
    let x = f128::from_bits(
        if e < 0 { a & SIGN } else { 0 } | ((e + 16383) as u128) << 112 | a & MASK | 1,
    );
    let scale = if e < -1 {
        e.max(-16382)
    } else {
        e.unsigned_abs().max(1).ilog2() as i32
    };
    let ey = (-scale - 116 + ((b >> 112) % 132) as i32).clamp(-16382, 16383);
    [
        x,
        f128::from_bits(b & SIGN | ((ey + 16383) as u128) << 112 | b & MASK),
    ]
}
fn near_midpoint(y: Float) -> bool {
    let r = y.to_f128();
    if !r.is_finite() || r == 0.0 {
        return false;
    }
    let b = r.to_bits();
    for next in [f128::from_bits(b - 1), f128::from_bits(b + 1)] {
        if !next.is_finite() {
            continue;
        }
        let gap = Float::with_val(PREC, next) - r;
        let mut delta = y.clone() - r;
        delta /= &gap;
        delta -= 0.5;
        if delta.abs() < 1.0 / 65536.0 {
            return true;
        }
    }
    false
}
fn main() {
    let count: u64 = std::env::args()
        .nth(1)
        .map(|s| s.parse().unwrap())
        .unwrap_or(5_000_000);
    let mut sections = vec![
        (
            "mathematical edges, exact powers, wide roots, half-ulp products".to_string(),
            edges(),
        ),
        (
            "inverse 114-bit midpoints, using expm1(log(z)/y)".to_string(),
            inverse(),
        ),
        (
            "plain full-domain sample".to_string(),
            (0..4000).map(|i| sample(0, i)).collect(),
        ),
    ];
    for kind in 0..4 {
        let mut found = std::thread::scope(|s| {
            let workers: Vec<_> = (0..8)
                .map(|t| {
                    s.spawn(move || {
                        let mut found = Vec::new();
                        for i in (t..count).step_by(8) {
                            let [x, y] = sample(kind, i);
                            let r = (Float::with_val(256, x).ln_1p() * y).exp();
                            if near_midpoint(r) {
                                found.push([x, y]);
                            }
                        }
                        found
                    })
                })
                .collect();
            workers
                .into_iter()
                .flat_map(|w| w.join().unwrap())
                .collect::<Vec<_>>()
        });
        found.sort_by_key(|[x, y]| (x.to_bits(), y.to_bits()));
        eprintln!(
            "band {kind}: {} near-midpoints / {count} scanned",
            found.len()
        );
        sections.push((
            format!("near-midpoint scan band {kind}, {count} pairs"),
            found,
        ));
    }
    let mut text = format!(
        "# compoundq(x,y) with correctly rounded MPFR answers.\n# CC=clang cargo +nightly run --release --features \"f128 mpfr\" --example gen_f128_compound_cases -- {count}\n"
    );
    let mut total = 0;
    for (label, mut cases) in sections {
        cases.sort_by_key(|[x, y]| (x.to_bits(), y.to_bits()));
        cases.dedup_by_key(|[x, y]| (x.to_bits(), y.to_bits()));
        writeln!(text, "#\n# {label}: {} cases\n#", cases.len()).unwrap();
        total += cases.len();
        for [x, y] in cases {
            writeln!(text, "{} {} {}", hex(x), hex(y), hex(cr_compound(x, y))).unwrap();
        }
    }
    std::fs::write("tests/cases/compoundq.wc", text).unwrap();
    eprintln!("wrote {total} cases");
}
