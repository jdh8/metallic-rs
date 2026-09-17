use crate::{common, common128};
use common::Identity as _;

#[cfg(feature = "mpfr")]
const SIGN: u128 = 1 << 127;
const MASK: u128 = (1 << 112) - 1;

#[cfg(feature = "mpfr")]
fn sample(i: u64) -> [f128; 2] {
    let a = common128::mix128(2 * i);
    let b = common128::mix128(2 * i + 1);
    let ex = match i % 4 {
        0 => i / 4 % 32767,
        1 => 16383 - 120 + i / 4 % 140,
        2 => 16383 - 19 + i / 4 % 20,
        _ => 0,
    } as i32;
    let x = f128::from_bits((ex as u128) << 112 | a & MASK | if ex < 16383 { a & SIGN } else { 0 });
    let e = ex.max(1) - 16383;
    let scale = if e < -1 {
        e
    } else {
        e.unsigned_abs().max(1).ilog2() as i32
    };
    let ey = (-scale - 116 + ((b >> 112) % 132) as i32).clamp(-16382, 16383);
    [
        x,
        f128::from_bits(b & SIGN | ((ey + 16383) as u128) << 112 | b & MASK),
    ]
}

#[test]
fn test_compoundq_special() {
    let inf = f128::INFINITY;
    let qnan = f128::NAN;
    let snan = f128::from_bits((0x7fff << 112) | 1);
    for x in [0.0, -0.0] {
        for y in [0.0, -0.0, 1.0, -1.0, inf, -inf, qnan] {
            assert!(metallic::compoundq(x, y).is(&1.0));
        }
        assert!(metallic::compoundq(x, snan).is_nan());
    }
    for y in [0.0, -0.0] {
        for x in [-1.0, -0.5, 1.0, inf, qnan] {
            assert!(metallic::compoundq(x, y).is(&1.0));
        }
        assert!(metallic::compoundq(snan, y).is_nan());
    }
    for x in [-2.0, -inf, f128::from_bits((-1.0_f128).to_bits() + 1)] {
        for y in [0.0, -0.0, 2.0, 3.0, inf, -inf] {
            assert!(metallic::compoundq(x, y).is_nan());
        }
    }
    for y in [0.5, 1.0, 2.0, inf] {
        assert!(metallic::compoundq(-1.0, y).is(&0.0));
        assert!(metallic::compoundq(-1.0, -y).is(&inf));
        assert!(metallic::compoundq(inf, y).is(&inf));
        assert!(metallic::compoundq(inf, -y).is(&0.0));
    }
    for x in [-0.5, 0.5] {
        assert!(metallic::compoundq(x, inf).is(&if x > 0.0 { inf } else { 0.0 }));
        assert!(metallic::compoundq(x, -inf).is(&if x > 0.0 { 0.0 } else { inf }));
        assert!(metallic::compoundq(x, qnan).is_nan());
        assert!(metallic::compoundq(qnan, x).is_nan());
    }
}

#[test]
fn test_compoundq_exact() {
    for (x, y, r) in [
        (0.5, 2.0, 2.25),
        (3.0, 0.5, 2.0),
        (80.0, 0.75, 27.0),
        (1.0, -16495.0, 0.0),
        (1.0, -16494.0, f128::from_bits(1)),
        (1.0, 16384.0, f128::INFINITY),
    ] {
        assert!(metallic::compoundq(x, y).is(&r));
    }
    let midpoint = 5_u128.pow(49);
    for (x, y) in [(4.0, 49.0), (24.0, 24.5), (624.0, 12.25)] {
        assert!(metallic::compoundq(x, y).is(&(midpoint as f128)));
    }
    // The square has 227 bits and is not representable, while square-1 is.
    // Its root 2^113+1 is an exact midpoint, and rounds down to even.
    let x = metallic::ldexpq(1.0, 226) + metallic::ldexpq(1.0, 114);
    assert!(metallic::compoundq(x, 0.5).is(&metallic::ldexpq(1.0, 113)));
    let x = metallic::ldexpq(1.0, 224) + metallic::ldexpq(1.0, 113);
    assert!(metallic::compoundq(x, 0.5).is(&(metallic::ldexpq(1.0, 112) + 1.0)));
    // The other tie direction: (2^114-1)^2 - 1 remains representable.
    let x = metallic::ldexpq(1.0, 228) - metallic::ldexpq(1.0, 115);
    assert!(metallic::compoundq(x, 0.5).is(&metallic::ldexpq(1.0, 114)));
}

#[test]
fn test_compoundq_vs_powq() {
    for i in 0..200_000 {
        let b = common128::mix128(i);
        let base = f128::from_bits((16382 + (b >> 126)) << 112 | b & MASK);
        let x = base - 1.0;
        if 1.0 + x != base {
            continue;
        }
        let y = (common::mix64(i) % 65536) as f128 / 16.0 - 2048.0;
        assert!(
            metallic::compoundq(x, y).is(&metallic::powq(base, y)),
            "x={x:?} y={y:?}"
        );
    }
}

#[test]
fn test_compoundq_vs_f32() {
    for i in 0..200_000 {
        let a = common::mix64(i);
        let x = f32::from_bits((a as u32 & 0x807f_ffff) | (((a >> 32) % 150) as u32) << 23);
        let y = ((a >> 40) as i32 - (1 << 23)) as f32 / 65536.0;
        let got = metallic::compoundq(x as f128, y as f128) as f32;
        let want = core_math::compoundf(x, y);
        assert!(
            got.is(&want)
                || (got.is_finite()
                    && want.is_finite()
                    && got.to_bits().abs_diff(want.to_bits()) <= 1),
            "x={x:?} y={y:?}"
        );
    }
}

#[test]
fn test_compoundq_corpus() {
    let cases: Vec<_> =
        common::parse_case_file("compoundq.wc", common128::parse_f128_triple).collect();
    assert_eq!(
        cases.len(),
        135_993,
        "corpus size changed; update this count"
    );
    common::truncate_errors(cases.into_iter().filter_map(|[x, y, want]| {
        let got = metallic::compoundq(x, y);
        (!got.is(&want)).then(|| println!("compoundq({x:?}, {y:?}) = {got:?} != {want:?}"))
    }));
}

#[cfg(feature = "mpfr")]
#[test]
fn test_compoundq_vs_mpfr() {
    common128::mpfr_sweep_bivariate_f128(
        metallic::compoundq,
        metallic::f128_mpfr::cr_compound,
        sample,
        400_000,
    );
    common128::mpfr_sweep_bivariate_f128(
        metallic::compoundq,
        metallic::f128_mpfr::cr_compound,
        |i| {
            [
                f128::from_bits(common128::mix128(i)),
                f128::from_bits(common128::mix128(i ^ 0xabcdef)),
            ]
        },
        200_000,
    );
}
