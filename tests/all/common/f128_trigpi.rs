#![allow(dead_code)]

use crate::common128;

pub const SIGN: u128 = 1 << 127;
pub const MASK: u128 = (1 << 112) - 1;
pub const FUNCTIONS: [fn(f128) -> f128; 3] = [metallic::sinpiq, metallic::cospiq, metallic::tanpiq];
pub const F64_FUNCTIONS: [fn(f64) -> f64; 3] =
    [core_math::sinpi, core_math::cospi, core_math::tanpi];

pub fn power(e: i32) -> f128 {
    if e < -16382 {
        f128::from_bits(1 << (e + 16494))
    } else {
        f128::from_bits(((e + 16383) as u128) << 112)
    }
}

pub fn band(i: u64, lo: i32, hi: i32) -> f128 {
    let bits = common128::mix128(i);
    let e = lo + ((bits >> 112 & 0x7fff) % (hi - lo + 1) as u128) as i32;
    f128::from_bits(bits & SIGN | ((e + 16383) as u128) << 112 | bits & MASK)
}

pub fn wide(i: u64) -> f128 {
    f128::from_bits(common128::mix128(i))
}

pub fn active(i: u64) -> f128 {
    band(i, -66, 112)
}

pub fn bench(i: u64) -> f128 {
    band(i, -20, 20)
}

/// Residuals about every table entry, including the exact zeros and poles,
/// from the polynomial band down through the precision of the argument.
pub fn near_grid(i: u64) -> f128 {
    let bits = common128::mix128(i);
    let base = ((bits >> 64) % 1024) as f128 / 256.0;
    let delta = power(-9 - (bits % 120) as i32);
    let x = if bits & 1 == 0 {
        base + delta
    } else {
        base - delta
    };
    if bits & SIGN == 0 { x } else { -x }
}

pub fn neighbors(out: &mut Vec<f128>, x: f128, radius: i128) {
    let b = x.to_bits() & !SIGN;
    if b >= f128::INFINITY.to_bits() {
        out.extend([x, -x]);
        return;
    }
    for d in -radius..=radius {
        if let Some(b) = b.checked_add_signed(d) {
            let x = f128::from_bits(b);
            out.extend([x, -x]);
        }
    }
}

/// The exact set, every table and rounding seam, the integer/half-integer
/// neighborhoods through 2^113, and every subnormal binade.
pub fn edges() -> Vec<f128> {
    let mut out = Vec::new();
    for x in [0.0, f128::INFINITY, f128::NAN, f128::MAX] {
        neighbors(&mut out, x, 2);
    }
    for e in (-16494..=-16380).chain([
        -67, -66, -65, -64, -63, -60, -59, -58, -10, -9, -8, -7, 103, 104, 105, 110, 111, 112, 113,
        114, 16383,
    ]) {
        neighbors(&mut out, power(e), 3);
    }
    for x in [1.0_f128 / 6.0, 1.0 / 3.0, 2.0 / 3.0, 5.0 / 6.0] {
        neighbors(&mut out, x, 3);
    }
    for j in 0..=1024 {
        neighbors(&mut out, j as f128 / 512.0, 2);
    }
    for e in 0..=113 {
        for j in 0..=8 {
            neighbors(&mut out, power(e) + j as f128 / 4.0, 3);
        }
    }
    out
}

#[cfg(feature = "mpfr")]
pub fn operation(kind: usize, y: &mut rug::Float) -> core::cmp::Ordering {
    use rug::float::Round::Nearest;
    match kind {
        0 => y.sin_pi_round(Nearest),
        1 => y.cos_pi_round(Nearest),
        2 => y.tan_pi_round(Nearest),
        _ => unreachable!(),
    }
}

#[cfg(feature = "mpfr")]
pub fn oracle(kind: usize, x: f128) -> f128 {
    metallic::f128_mpfr::cr_unop(x, |y| operation(kind, y))
}

pub fn corpus(name: &str, count: usize, kind: usize) {
    let cases: Vec<[f128; 2]> =
        crate::common::parse_case_file(format!("{name}.wc"), common128::parse_f128_pair).collect();
    assert_eq!(cases.len(), count, "corpus size changed; update the count");
    use crate::common::Identity as _;
    crate::common::truncate_errors(cases.into_iter().filter_map(|[x, want]| {
        let got = FUNCTIONS[kind](x);
        (!got.is(&want)).then(|| println!("{name}({x:?}) = {got:?} != {want:?}"))
    }));
}

pub fn cross_check(kind: usize) {
    crate::common::truncate_errors((0..200_000).filter_map(|i| {
        let x = f64::from_bits(crate::common::mix64(i));
        let got = FUNCTIONS[kind](x as f128) as f64;
        let want = F64_FUNCTIONS[kind](x);
        (crate::common::ulp_error_f64(got, want) > 1)
            .then(|| println!("kind={kind}, x={x:e}: {got:e} != {want:e}"))
    }));
}

#[cfg(feature = "mpfr")]
pub fn sweep(kind: usize) {
    let f = FUNCTIONS[kind];
    let cr = |x| oracle(kind, x);
    crate::common::test_univariate_cases(f, cr, edges().into_iter());
    // Every exponent field gets eight independent significands and signs.
    let binades = (0..0x7fff_u64 * 8).map(|i| {
        let bits = common128::mix128(i);
        f128::from_bits(bits & SIGN | u128::from(i / 8) << 112 | bits & MASK)
    });
    crate::common::test_univariate_cases(f, cr, binades);
    for sampler in [wide, active, bench, near_grid] {
        common128::mpfr_sweep_univariate_f128(f, cr, sampler, 200_000);
    }
}

/// Odd sine/tangent and even cosine on the complete representation space.
pub fn symmetry(kind: usize) {
    use crate::common::Identity as _;
    let f = FUNCTIONS[kind];
    crate::common::truncate_errors((0..200_000).filter_map(|i| {
        let x = wide(i);
        let want = if kind == 1 { f(x) } else { -f(x) };
        let got = f(-x);
        (!got.is(&want)).then(|| println!("kind={kind}, x={x:?}: symmetry {got:?} != {want:?}"))
    }));
}
