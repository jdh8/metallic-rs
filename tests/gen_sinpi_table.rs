//! Throwaway generator for the table-driven `abs_sinpi_dd_lean`.  Prints
//! `SINPI_TABLE[k] = sin(πk/128)` (k = 0..=64) as double-double and the
//! Taylor coefficients of `cos(πd/128)` / `sin(πd/128)` in grid units `d`.
#![cfg(feature = "mpfr")]
use rug::Float;
use rug::float::Constant;

fn dd(x: &Float) -> (f64, f64) {
    let hi = x.to_f64();
    let lo = (x.clone() - hi).to_f64();
    (hi, lo)
}

/// Run on demand: `cargo test --release --features mpfr --test gen_sinpi_table -- --ignored --nocapture`
#[test]
#[ignore = "generator, not a test — prints the SINPI_TABLE source"]
fn generate() {
    let prec = 200;
    let pi = Float::with_val(prec, Constant::Pi);

    println!("// SINPI_TABLE[k] = sin(πk/128), k = 0..=64");
    println!("const SINPI_TABLE: [DoubleDouble; 65] = [");
    for k in 0..=64u32 {
        let arg = pi.clone() * Float::with_val(prec, k) / Float::with_val(prec, 128);
        let (hi, lo) = dd(&arg.sin());
        println!("    DoubleDouble {{ high: {hi:?}, low: {lo:?} }},");
    }
    println!("];");

    // cos(πd/128) = 1 + d²(C1 + d²(C2 + d²·C3)),  C_k = (-1)^k (π/128)^{2k}/(2k)!
    // sin(πd/128) = d (S0 + d²(S1 + d²(S2 + d²·S3))), S_k = (-1)^k (π/128)^{2k+1}/(2k+1)!
    let s = pi.clone() / Float::with_val(prec, 128); // π/128
    let fact = |n: u32| {
        let mut f = Float::with_val(prec, 1);
        for i in 2..=n {
            f *= Float::with_val(prec, i);
        }
        f
    };
    let pow = |n: u32| {
        let mut p = Float::with_val(prec, 1);
        for _ in 0..n {
            p *= &s;
        }
        p
    };
    // Leading C1, S0 as dd; the rest as f64 high words.
    let c1 = -(pow(2) / fact(2));
    let (c1h, c1l) = dd(&c1);
    let c2 = (pow(4) / fact(4)).to_f64();
    let c3 = -(pow(6) / fact(6)).to_f64();
    let s0 = s.clone();
    let (s0h, s0l) = dd(&s0);
    let s1 = -(pow(3) / fact(3));
    let (s1h, s1l) = dd(&s1);
    let s2 = (pow(5) / fact(5)).to_f64();
    let s3 = -(pow(7) / fact(7)).to_f64();
    println!("// cos: C1 dd, C2 C3 f64");
    println!("const SINPI_C1: DoubleDouble = DoubleDouble {{ high: {c1h:?}, low: {c1l:?} }};");
    println!("const SINPI_C2: f64 = {c2:?};");
    println!("const SINPI_C3: f64 = {c3:?};");
    println!("// sin: S0 dd, S1 dd, S2 S3 f64");
    println!("const SINPI_S0: DoubleDouble = DoubleDouble {{ high: {s0h:?}, low: {s0l:?} }};");
    println!("const SINPI_S1: DoubleDouble = DoubleDouble {{ high: {s1h:?}, low: {s1l:?} }};");
    println!("const SINPI_S2: f64 = {s2:?};");
    println!("const SINPI_S3: f64 = {s3:?};");
}
