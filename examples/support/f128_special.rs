//! Shared, kernel-independent support for the erf/gamma corpus generators.
#![allow(dead_code)]
use rug::{Float, float::Round};
use std::fmt::Write as _;
pub const PREC: u32 = 512;
// Forty-seven guard bits suffice to select a 2^-14 midpoint window in a
// twenty-million-input scan. Frozen answers are evaluated afresh at 113 bits
// with MPFR's ternary result; scan approximations are never used as answers.
pub const SCAN_PREC: u32 = 160;
pub const SIGN: u128 = 1 << 127;
pub const MASK: u128 = (1 << 112) - 1;

pub fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}
pub fn bits(i: u64) -> u128 {
    u128::from(mix(i)) | u128::from(mix(i ^ 0x9e37_79b9_7f4a_7c15)) << 64
}
pub fn power(e: i32) -> Float {
    Float::with_val(PREC, 1) << e
}
pub fn neighbors(out: &mut Vec<f128>, x: f128, radius: u128) {
    if !x.is_finite() {
        return;
    }
    let b = x.to_bits() & !SIGN;
    for k in b.saturating_sub(radius)..=b.saturating_add(radius) {
        if k < f128::INFINITY.to_bits() {
            out.extend([f128::from_bits(k), f128::from_bits(k | SIGN)]);
        }
    }
}
pub fn operation(kind: usize, x: &mut Float) -> std::cmp::Ordering {
    match kind {
        0 => x.erf_round(Round::Nearest),
        1 => x.erfc_round(Round::Nearest),
        2 => x.gamma_round(Round::Nearest),
        _ => x.ln_abs_gamma_round(Round::Nearest).1,
    }
}
pub fn hex(x: f128) -> String {
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
pub fn domain(i: u64) -> f128 {
    let b = bits(i);
    f128::from_bits(b & SIGN | (i as u128 % 0x8000) << 112 | b & MASK)
}
pub fn common_edges() -> Vec<f128> {
    let mut out = vec![
        0.0,
        -0.0,
        f128::NAN,
        -f128::NAN,
        f128::INFINITY,
        f128::NEG_INFINITY,
    ];
    for e in -16494..=16383 {
        let x = metallic::ldexpq(1.0, e);
        out.extend([x, -x]);
        if e % 128 == 0
            || [
                -16382, -280, -256, -128, -114, -113, -112, -57, -1, 0, 1, 6, 7, 112,
            ]
            .contains(&e)
        {
            neighbors(&mut out, x, 4);
        }
    }
    for b in 0..256 {
        out.extend([f128::from_bits(b), f128::from_bits(b | SIGN)]);
    }
    neighbors(&mut out, f128::MAX, 4);
    out
}
pub fn midpoint_distance(y: Float) -> f64 {
    let rounded = y.to_f128_round(Round::Nearest);
    if !rounded.is_finite() || rounded == 0.0 {
        return 1.0;
    }
    let c = Float::with_val(SCAN_PREC, rounded);
    let other = if y > c {
        rounded.next_up()
    } else {
        rounded.next_down()
    };
    if !other.is_finite() {
        return 1.0;
    }
    let gap = (Float::with_val(SCAN_PREC, other) - &c).abs();
    (1.0 - 2.0 * ((y - c).abs() / gap).to_f64()).abs()
}
pub fn scan(kind: usize, count: u64, sample: fn(u64) -> f128) -> Vec<f128> {
    let threads: u64 = std::env::var("CORPUS_THREADS")
        .ok()
        .map_or(2, |s| s.parse().unwrap());
    let mut kept = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|thread| {
                scope.spawn(move || {
                    let mut out = Vec::new();
                    for i in (thread..count).step_by(threads as usize) {
                        let x = sample(i);
                        let mut y = Float::with_val(SCAN_PREC, x);
                        operation(kind, &mut y);
                        if midpoint_distance(y) < 1.0 / 16384.0 {
                            out.push(x);
                        }
                        if thread == 0 && i % 1_000_000 == 0 {
                            eprintln!("kind={kind}: scanned {i}/{count}");
                        }
                    }
                    out
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().unwrap())
            .collect::<Vec<_>>()
    });
    kept.sort_by_key(|x| x.to_bits());
    eprintln!(
        "kind={kind}: {} near-midpoints / {count} inputs",
        kept.len()
    );
    kept
}
pub fn write_corpus(kind: usize, command: &str, sections: Vec<(&str, Vec<f128>)>) {
    let name = ["erfq", "erfcq", "tgammaq", "lgammaq"][kind];
    let mut text = format!(
        "# {name}(x): MPFR precision-113 answers with ternary-aware IEEE subnormalization.\n# {command}\n"
    );
    let mut seen = std::collections::HashSet::new();
    for (title, cases) in sections {
        writeln!(text, "#\n# {title}").unwrap();
        for x in cases {
            if !seen.insert(x.to_bits()) {
                continue;
            }
            let y = metallic::f128_mpfr::cr_unop(x, |y| operation(kind, y));
            writeln!(text, "{} {}", hex(x), hex(y)).unwrap();
        }
    }
    let path = format!("tests/cases/{name}.wc");
    std::fs::write(&path, text).unwrap();
    eprintln!("wrote {} cases to {path}", seen.len());
}
