//! `fmaq` is the platform's binary128 fused multiply-add (glibc's `fmaf128`
//! where `long double` is binary128).  Metallic does not implement it, so this
//! gate is a contract check rather than a rounding proof: the single-rounding
//! property on exact cases, and the MPFR cross-check where the oracle is
//! available.
use crate::common128;
use metallic::fmaq;

const SAMPLES: u64 = 100_000;

/// The defining property: one rounding, not two.  `(1 + ε)² − 1` is exactly
/// `2ε + ε²`, which a rounded multiply loses and a fused one keeps.
#[test]
fn single_rounding() {
    let one_plus = 1.0 + f128::EPSILON;
    assert_eq!(
        fmaq(one_plus, one_plus, -1.0),
        2.0 * f128::EPSILON + f128::EPSILON * f128::EPSILON
    );

    // Exact small integers are exact either way.
    assert_eq!(fmaq(3.0_f128, 4.0, 5.0), 17.0);
    assert_eq!(fmaq(-3.0_f128, 4.0, 5.0), -7.0);
}

/// `fmaq(a, b, -(a * b))` is the error-free transform's residual: adding it
/// back to the rounded product reproduces `a * b` exactly, which only a true
/// fused operation can do.
#[test]
fn two_product() {
    for i in 0..SAMPLES {
        // Keep both operands in a binade where the product cannot overflow or
        // reach the subnormals, so the transform is exact by Dekker's theorem.
        let bits = common128::mix128(i);
        let head = |b: u128| f128::from_bits(b & SIGN | 0x3fff << 112 | b & MANTISSA);
        let (a, b) = (head(bits), head(bits >> 1 | bits << 127));

        let product = a * b;
        let residual = fmaq(a, b, -product);
        assert_eq!(fmaq(a, b, -product) - residual, 0.0);
        assert!(
            residual.abs() <= product.abs() * f128::EPSILON,
            "fmaq({a:?}, {b:?}, ...)"
        );
        // The residual is representable, so the split is lossless.
        assert_eq!(product + residual, product);
    }
}

const SIGN: u128 = 1 << 127;
const MANTISSA: u128 = (1 << 112) - 1;

#[cfg(feature = "mpfr")]
#[test]
fn test_fmaq_vs_mpfr() {
    use rug::float::Round::Nearest;

    for i in 0..SAMPLES {
        let sample = |k: u64| f128::from_bits(common128::mix128(i ^ k));
        let (x, y, a) = (sample(0), sample(0x5bf0_3635), sample(0xc2b2_ae35));

        let want = metallic::f128_mpfr::cr_terop(x, y, a, |p, q, r| p.mul_add_round(q, r, Nearest));
        let got = fmaq(x, y, a);
        assert!(
            got.to_bits() == want.to_bits() || (got.is_nan() && want.is_nan()),
            "fmaq({x:?}, {y:?}, {a:?}) = {got:?} != {want:?}"
        );
    }
}
