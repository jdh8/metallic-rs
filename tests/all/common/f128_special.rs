//! Binary128 erf/gamma samples, corpus gates, and independent MPFR checks.
#![allow(dead_code)]
use super::{common, common128};
use common::Identity as _;

pub const SIGN: u128 = 1 << 127;
pub const MASK: u128 = (1 << 112) - 1;
pub const FUNCTIONS: [fn(f128) -> f128; 4] = [
    metallic::erfq,
    metallic::erfcq,
    metallic::tgammaq,
    metallic::lgammaq,
];
const SAMPLES: u64 = 131_072;

pub fn domain(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | (i as u128 % 0x8000) << 112 | b & MASK)
}
pub fn subnormal(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | b & ((1 << 114) - 1))
}
pub fn tiny(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | (16383 - 280 + i as u128 % 282) << 112 | b & MASK)
}
pub fn erf_band(i: u64) -> f128 {
    let b = common128::mix128(i);
    f128::from_bits(b & SIGN | (16383 - 8 + i as u128 % 15) << 112 | b & MASK)
}
pub fn erfc_tail(i: u64) -> f128 {
    let b = common128::mix128(i);
    (b & MASK) as f128 * metallic::ldexpq(108.0, -112)
}
pub fn erf_seams(i: u64) -> f128 {
    let e = (i / 16 % 8) as i32 - 1;
    let x = metallic::ldexpq((16 + i % 16) as f128, e - 4);
    let b = common128::mix128(i);
    f128::from_bits(
        b & SIGN
            | x.to_bits()
                .wrapping_add_signed((i / 128 % 1025) as i128 - 512),
    )
}
pub fn gamma_band(i: u64) -> f128 {
    let b = common128::mix128(i);
    let x = (b & MASK) as f128 * metallic::ldexpq(1800.0, -112);
    if b & SIGN == 0 { x } else { -x }
}
pub fn gamma_poles(i: u64) -> f128 {
    let b = common128::mix128(i);
    let n = (i % 1800 + 1) as f128;
    let delta = (b & ((1_u128 << (1 + i / 1800 % 112)) - 1)).max(1);
    -f128::from_bits(if i % 2 == 0 {
        n.to_bits() + delta
    } else {
        n.to_bits() - delta
    })
}
pub fn gamma_roots(i: u64) -> f128 {
    let b = common128::mix128(i);
    let center = if i % 2 == 0 { 1.0_f128 } else { 2.0 };
    let delta = (b & ((1_u128 << (1 + i / 2 % 112)) - 1)).max(1);
    f128::from_bits(if i % 4 < 2 {
        center.to_bits() + delta
    } else {
        center.to_bits() - delta
    })
}
pub fn gamma_seams(i: u64) -> f128 {
    let x = match i % 7 {
        0 => 1.0 + (i / 7 % 17) as f128 / 16.0,
        1 => (i / 7 % 1800 + 1) as f128,
        2 => 64.0,
        3 => 0.5,
        4 => 2048.0,
        5 => 31.0 / 32.0,
        _ => metallic::ldexpq(1.0, -256),
    };
    let b = common128::mix128(i);
    f128::from_bits(
        b & SIGN
            | x.to_bits()
                .wrapping_add_signed((i / 128 % 1025) as i128 - 512),
    )
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
    match kind {
        0 => {
            assert!(f(0.0).is(&0.0));
            assert!(f(-0.0).is(&-0.0));
            assert_eq!(f(f128::INFINITY), 1.0);
            assert_eq!(f(f128::NEG_INFINITY), -1.0);
            assert_eq!(f(f128::MAX), 1.0);
            assert_eq!(f(-f128::MAX), -1.0);
        }
        1 => {
            assert_eq!(f(0.0), 1.0);
            assert_eq!(f(-0.0), 1.0);
            assert_eq!(f(f128::INFINITY).to_bits(), 0);
            assert_eq!(f(f128::NEG_INFINITY), 2.0);
            assert_eq!(f(f128::MAX).to_bits(), 0);
            assert_eq!(f(-f128::MAX), 2.0);
            assert!(f(106.0) > 0.0);
            assert_eq!(f(108.0).to_bits(), 0);
        }
        2 => {
            assert_eq!(f(0.0), f128::INFINITY);
            assert_eq!(f(-0.0), f128::NEG_INFINITY);
            assert_eq!(f(f128::INFINITY), f128::INFINITY);
            assert!(f(f128::NEG_INFINITY).is_nan());
            assert_eq!(f(f128::MAX), f128::INFINITY);
            for n in 1..=1024 {
                assert!(f(-(n as f128)).is_nan());
            }
            assert!(f(-f128::MAX).is_nan());
            let mut factorial = 1_u128;
            for n in 1..=34 {
                assert_eq!(f(n as f128).to_bits(), (factorial as f128).to_bits());
                factorial = factorial.checked_mul(n).unwrap();
            }
        }
        _ => {
            for x in [
                0.0,
                -0.0,
                f128::INFINITY,
                f128::NEG_INFINITY,
                f128::MAX,
                -f128::MAX,
            ] {
                assert_eq!(f(x), f128::INFINITY);
            }
            for n in 1..=1024 {
                assert_eq!(f(-(n as f128)), f128::INFINITY);
            }
            assert_eq!(f(1.0).to_bits(), 0);
            assert_eq!(f(2.0).to_bits(), 0);
            assert!(f(1.5) < 0.0);
        }
    }
}
pub fn identities(kind: usize) {
    let f = FUNCTIONS[kind];
    for i in 0..4096 {
        if kind < 2 {
            let x = erf_band(i).abs();
            let y = f(x);
            if kind == 0 {
                assert_eq!(f(-x).to_bits(), y.to_bits() | SIGN);
            }
            if kind == 0 {
                assert!(f(x.next_down()) <= y && y <= f(x.next_up()));
            } else {
                assert!(f(x.next_down()) >= y && y >= f(x.next_up()));
            }
        } else {
            let x = gamma_band(i).abs() + 2.0;
            let y = f(x);
            assert!(f(x.next_down()) <= y && y <= f(x.next_up()));
        }
    }
}
pub fn vs_f64(kind: usize) {
    let reference = [
        core_math::erf,
        core_math::erfc,
        core_math::tgamma,
        core_math::lgamma,
    ][kind];
    common::truncate_errors((0..32768).filter_map(|i| {
        let x = if i % 2 == 0 {
            f64::from_bits(common::mix64(i))
        } else if kind < 2 {
            erf_band(i) as f64
        } else {
            gamma_band(i) as f64
        };
        let got = FUNCTIONS[kind](x as f128) as f64;
        let want = reference(x);
        (common::ulp_error_f64(got, want) > 1)
            .then(|| println!("special {kind}({x:e}): {got:e} != {want:e}"))
    }));
}
#[cfg(feature = "mpfr")]
pub fn operation(kind: usize, x: &mut rug::Float) -> std::cmp::Ordering {
    use rug::float::Round::Nearest;
    match kind {
        0 => x.erf_round(Nearest),
        1 => x.erfc_round(Nearest),
        2 => x.gamma_round(Nearest),
        _ => x.ln_abs_gamma_round(Nearest).1,
    }
}
#[cfg(feature = "mpfr")]
pub fn oracle(kind: usize, x: f128) -> f128 {
    metallic::f128_mpfr::cr_unop(x, |y| operation(kind, y))
}
#[cfg(feature = "mpfr")]
pub fn vs_mpfr(kind: usize) {
    let samples: [fn(u64) -> f128; 7] = if kind < 2 {
        [
            domain, subnormal, tiny, erf_band, erfc_tail, erf_seams, domain,
        ]
    } else {
        [
            domain,
            subnormal,
            tiny,
            gamma_band,
            gamma_poles,
            gamma_roots,
            gamma_seams,
        ]
    };
    for sample in samples {
        common128::mpfr_sweep_univariate_f128(
            FUNCTIONS[kind],
            |x| oracle(kind, x),
            sample,
            SAMPLES,
        );
    }
}
