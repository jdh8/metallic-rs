use super::exp::finite_exp;
use crate::f64::double::{round, round_general, DoubleDouble};

/// `erf(x)` for `|x| < 0.4375` as `x·P(x²)`
///
/// The minimax coefficients (from CORE-MATH) and this evaluation order are
/// chosen so that the single `f64` product rounds correctly to `f32`; `c[0]` is
/// the correctly-rounded `2/√π`, keeping the tiny-argument regime exact.
#[inline]
fn erf_small(x: f64) -> f64 {
    const C: [f64; 8] = [
        1.128_379_167_095_512_6,
        -0.376_126_389_031_781_8,
        0.112_837_916_703_424_2,
        -0.026_866_170_388_309_935,
        0.005_223_972_335_150_932_5,
        -0.000_854_773_440_605_154_9,
        0.000_120_184_475_094_822_11,
        -1.372_114_526_702_553_9e-5,
    ];

    let z2 = x * x;
    let z4 = z2 * z2;
    let z8 = z4 * z4;
    let c0 = z4.mul_add(z2.mul_add(C[3], C[2]), z2.mul_add(C[1], C[0]));
    let c4 = z4.mul_add(z2.mul_add(C[7], C[6]), z2.mul_add(C[5], C[4]));

    x * z8.mul_add(c4, c0)
}

/// `erfc(x)` as a double-double for finite `x` in `[0.4375, 0x1.41bbf8p+3]`
///
/// Uses `erfc(x) = (2/(2+x))·exp(Q(t) - x²)` with `t = 2/(2+x)`, where the
/// degree-22 polynomial `Q(t)` approximates `x² + ln(erfc(x)·(2+x)/2)` — a
/// smooth, `O(1)` function.  `x²` is formed exactly and the exponent is carried
/// in double-double so the single-rounding error of `exp` dominates.
///
/// `Q(t)` was generated with
/// `ratapprox --function="(2/x-2)^2 + log(erfc(2/x-2)) - log(x)"
///  --dom="[0.16,0.8205]" --num="[1,x,...,x^22]" --den="[1]" --weight=1`.
fn erfc_dd(x: f64) -> DoubleDouble {
    const Q: [f64; 23] = [
        -1.265_512_122_089_828_2,
        0.999_999_909_841_198_7,
        0.375_002_743_471_048_67,
        0.083_281_051_140_198_57,
        -0.085_237_288_049_587_05,
        -0.150_760_071_598_243_3,
        -0.037_328_341_350_081_02,
        -0.307_000_710_902_394_2,
        1.813_760_492_069_090_2,
        -6.732_827_419_146_119,
        22.932_196_277_953_373,
        -63.486_057_493_716_345,
        143.005_106_419_187_14,
        -264.328_775_383_285_2,
        394.457_961_771_325_7,
        -465.717_326_957_287_7,
        428.276_437_921_645_1,
        -302.337_694_534_648_9,
        160.625_086_292_286_43,
        -62.167_646_608_866_96,
        16.552_127_045_415_844,
        -2.710_346_863_587_057_3,
        0.205_553_868_706_903_5,
    ];

    let t = DoubleDouble::from_quotient(2.0, 2.0 + x);
    let q = crate::poly(t.high, &Q);

    // exponent W = Q(t) - x², with x² exact and the difference in double-double
    let x2 = DoubleDouble::from_product(x, x);
    let w = DoubleDouble { high: q, low: 0.0 }
        + DoubleDouble {
            high: -x2.high,
            low: -x2.low,
        };

    // exp(W) = exp(W.high)·exp(W.low) ≈ exp(W.high)·(1 + W.low)
    t * (finite_exp(w.high) * (1.0 + w.low))
}

/// The error function
#[must_use]
#[inline]
pub fn erf(x: f32) -> f32 {
    if x.is_nan() {
        return x;
    }

    let ax = x.abs();

    if ax < 0.437_5 {
        return erf_small(x.into()) as f32;
    }

    // erf rounds to ±1 for |x| > 0x1.f5a888p+1 (covers ∞)
    let magnitude = if ax > f32::from_bits(0x407a_d444) {
        1.0
    } else {
        let e = erfc_dd(ax.into());
        round(
            DoubleDouble {
                high: 1.0,
                low: 0.0,
            } + DoubleDouble {
                high: -e.high,
                low: -e.low,
            },
        )
    };

    magnitude.copysign(x)
}

/// The complementary error function `1 - erf(x)`
#[must_use]
#[inline]
pub fn erfc(x: f32) -> f32 {
    if x.is_nan() {
        return x;
    }

    let ax = x.abs();

    if ax < 0.437_5 {
        // Lone hard-to-round case in this regime (found by exhaustive sweep):
        // erfc(-0x1.d93ec4p-17) sits a hair past an f32 midpoint.
        if x.to_bits() == 0xb76c_9f62 {
            return f32::from_bits(0x3f80_0085);
        }
        return (1.0 - erf_small(x.into())) as f32;
    }

    // erfc underflows to 0 for large x; the reflection 2 - erfc tends to 2
    if ax >= f32::from_bits(0x4120_ddfc) {
        return if x.is_sign_positive() { 0.0 } else { 2.0 };
    }

    let e = erfc_dd(ax.into());

    if x.is_sign_positive() {
        round_general(e)
    } else {
        round_general(
            DoubleDouble {
                high: 2.0,
                low: 0.0,
            } + DoubleDouble {
                high: -e.high,
                low: -e.low,
            },
        )
    }
}
