//! Samples shared by the binary128 hyperbolic correctness gates.
#![allow(dead_code)]
use super::{common, common128};
use common::Identity as _;

pub const SIGN: u128 = 1 << 127;
pub const MASK: u128 = (1 << 112) - 1;
pub const FUNCTIONS: [fn(f128) -> f128; 3] = [metallic::sinhq, metallic::coshq, metallic::tanhq];
const SAMPLES: u64 = 200_000;

pub fn domain(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | (i as u128 % 0x8000) << 112 | b & MASK)
}
pub fn band(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | (16383 - 58 + i as u128 % 73) << 112 | b & MASK)
}
pub fn subnormal(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | b & ((1 << 114) - 1))
}
pub fn seams(i: u64) -> f128 {
    let b = common128::mix128(i);
    let e = [-57, -56, -17, -16, -5, -4, 6, 14][(i / 2 % 8) as usize];
    let anchor = ((16383 + e) as u128) << 112;
    let offset = (i / 16) as i128 - (SAMPLES / 32) as i128;
    f128::from_bits(b & SIGN | anchor.wrapping_add_signed(offset))
}
pub fn dense(i: u64) -> f128 {
    let b = common128::mix128(i);
    let a = (b & MASK) as f128 * metallic::ldexpq(1.0, -112) * 11400.0;
    if b & SIGN != 0 { -a } else { a }
}
pub fn saturation(i: u64) -> f128 {
    let b = common128::mix128(i);
    let a = 30.0 + (b & MASK) as f128 * metallic::ldexpq(1.0, -112) * 15.0;
    if b & SIGN != 0 { -a } else { a }
}

pub fn corpus(kind: usize, name: &str, count: usize) {
    let cases: Vec<[f128; 2]> = common::parse_case_file(name, common128::parse_f128_pair).collect();
    assert!(count > 0, "the MPFR corpus must exist");
    assert_eq!(cases.len(), count, "corpus size changed; update the count");
    common::truncate_errors(cases.into_iter().filter_map(|[x, want]| {
        let got = FUNCTIONS[kind](x);
        (!got.is(&want)).then(|| println!("{name}: f({x:?}) = {got:?} != {want:?}"))
    }));
}
pub fn special(kind: usize) {
    let f = FUNCTIONS[kind];
    for x in [0.0_f128, -0.0] {
        assert!(f(x).is(&if kind == 1 { 1.0 } else { x }));
    }
    for x in [f128::INFINITY, f128::NEG_INFINITY, f128::MAX, -f128::MAX] {
        let want = match kind {
            0 => f128::INFINITY.copysign(x),
            1 => f128::INFINITY,
            _ => 1.0_f128.copysign(x),
        };
        assert!(f(x).is(&want));
    }
    for bits in [
        0x7fff_u128 << 112 | 1,
        0xffff_u128 << 112 | 7,
        f128::NAN.to_bits(),
    ] {
        assert_eq!(f(f128::from_bits(bits)).to_bits(), bits | 1 << 111);
    }
    for x in [
        f128::from_bits(1),
        f128::MIN_POSITIVE,
        metallic::ldexpq(1.0, -57),
    ] {
        for x in [x, -x] {
            assert!(f(x).is(&if kind == 1 { 1.0 } else { x }));
        }
    }
    // At 2^-56 the cosh square is exactly a half ulp above 1;
    // its positive fourth-order term breaks the tie upward.
    if kind == 1 {
        assert_eq!(f(metallic::ldexpq(1.0, -56)), 1.0_f128.next_up());
    }
}
pub fn vs_f64(kind: usize) {
    let oracle = [core_math::sinh, core_math::cosh, core_math::tanh][kind];
    common::truncate_errors((0..SAMPLES).filter_map(|i| {
        let x = f64::from_bits(common::mix64(i));
        let got = FUNCTIONS[kind](x as f128) as f64;
        let want = oracle(x);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("hyperbolic {kind}({x:e}): {got:e} != {want:e}"))
    }));
}
pub fn symmetry_and_monotonicity(kind: usize) {
    let f = FUNCTIONS[kind];
    for i in 0..20_000 {
        let x = band(i).abs();
        let y = f(x);
        assert_eq!(
            f(-x).to_bits(),
            y.to_bits() | if kind == 1 { 0 } else { SIGN }
        );
        assert!(f(x.next_down()) <= y && y <= f(x.next_up()));
    }
}
#[cfg(feature = "mpfr")]
pub fn operation(kind: usize, y: &mut rug::Float) -> core::cmp::Ordering {
    use rug::float::Round::Nearest;
    match kind {
        0 => y.sinh_round(Nearest),
        1 => y.cosh_round(Nearest),
        _ => y.tanh_round(Nearest),
    }
}
#[cfg(feature = "mpfr")]
pub fn vs_mpfr(kind: usize) {
    for sample in [domain, band, subnormal, seams, dense, saturation] {
        common128::mpfr_sweep_univariate_f128(
            FUNCTIONS[kind],
            |x| metallic::f128_mpfr::cr_unop(x, |y| operation(kind, y)),
            sample,
            SAMPLES,
        );
    }
}
