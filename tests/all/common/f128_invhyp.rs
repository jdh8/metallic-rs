//! Samples shared by the binary128 inverse-hyperbolic gates.
#![allow(dead_code)]
use super::{common, common128};
use common::Identity as _;
const SIGN: u128 = 1 << 127;
const MASK: u128 = (1 << 112) - 1;
const ONE: u128 = 16383 << 112;
const SAMPLES: u64 = 200_000;
pub const FUNCTIONS: [fn(f128) -> f128; 2] = [metallic::asinhq, metallic::acoshq];

pub fn domain(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | (i as u128 % 0x8000) << 112 | b & MASK)
}
pub fn band(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | (16383 - 58 + i as u128 % 252) << 112 | b & MASK)
}
pub fn positive(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits((16383 + i as u128 % 16384) << 112 | b & MASK)
}
pub fn near_one(i: u64) -> f128 {
    let b = common128::mix128(i);
    let delta = (b & ((1_u128 << (1 + i % 113)) - 1)).max(1);
    f128::from_bits(if i % 4 == 0 { ONE - delta } else { ONE + delta })
}
pub fn subnormal(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | b & ((1 << 114) - 1))
}
pub fn seams(i: u64) -> f128 {
    let b = common128::mix128(i);
    let anchors = [-57, -56, -17, -16, -5, -4, 0, 1, 79, 80, 191, 192];
    let index = (i / 2 % 13) as usize;
    let anchor = if index == 12 {
        ONE + (1 << 105)
    } else {
        ((16383 + anchors[index]) as u128) << 112
    };
    let offset = (i / 26) as i128 - (SAMPLES / 52) as i128;
    f128::from_bits(b & SIGN | anchor.wrapping_add_signed(offset))
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
    for bits in [
        0x7fff_u128 << 112 | 1,
        0xffff_u128 << 112 | 7,
        f128::NAN.to_bits(),
    ] {
        assert_eq!(f(f128::from_bits(bits)).to_bits(), bits | 1 << 111);
    }
    assert_eq!(f(f128::INFINITY), f128::INFINITY);
    assert!(f(f128::MAX).is_finite());
    if kind == 0 {
        for x in [
            0.0_f128,
            -0.0,
            f128::from_bits(1),
            f128::MIN_POSITIVE,
            metallic::ldexpq(1.0, -57),
        ] {
            assert!(f(x).is(&x));
            assert!(f(-x).is(&-x));
        }
        assert_eq!(f(f128::NEG_INFINITY), f128::NEG_INFINITY);
    } else {
        for x in [
            0.0_f128,
            -0.0,
            -1.0,
            1.0_f128.next_down(),
            f128::NEG_INFINITY,
            -f128::MAX,
        ] {
            assert!(f(x).is_nan());
        }
        assert_eq!(f(1.0).to_bits(), 0);
        assert!(f(1.0_f128.next_up()) > 0.0);
    }
}
pub fn vs_f64(kind: usize) {
    let oracle = [core_math::asinh, core_math::acosh][kind];
    common::truncate_errors((0..SAMPLES).filter_map(|i| {
        let mut x = f64::from_bits(common::mix64(i));
        if kind == 1 && i % 2 == 0 {
            x = 1.0 + x.abs();
        }
        let got = FUNCTIONS[kind](x as f128) as f64;
        let want = oracle(x);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("inverse hyperbolic {kind}({x:e}): {got:e} != {want:e}"))
    }));
}
pub fn symmetry_and_monotonicity(kind: usize) {
    let f = FUNCTIONS[kind];
    for i in 0..20_000 {
        let x = if kind == 0 {
            band(i).abs()
        } else if i % 2 == 0 {
            near_one(i).max(1.0_f128.next_up())
        } else {
            positive(i)
        };
        let y = f(x);
        if kind == 0 {
            assert_eq!(f(-x).to_bits(), y.to_bits() | SIGN);
        }
        assert!(f(x.next_down()) <= y && y <= f(x.next_up()));
    }
}
#[cfg(feature = "mpfr")]
pub fn vs_mpfr(kind: usize) {
    use rug::float::Round::Nearest;
    for sample in [domain, band, positive, near_one, subnormal, seams] {
        common128::mpfr_sweep_univariate_f128(
            FUNCTIONS[kind],
            |x| {
                metallic::f128_mpfr::cr_unop(x, |y| {
                    if kind == 0 {
                        y.asinh_round(Nearest)
                    } else {
                        y.acosh_round(Nearest)
                    }
                })
            },
            sample,
            SAMPLES,
        );
    }
}
