mod common;
use metallic::f64 as metal;

#[test]
fn test_log() {
    // `log(x, base) = log2(x) / log2(base)` is only faithfully rounded, so check a
    // tight tolerance against the std reference, plus a few cases that are exact
    // because `log2` of a power of two is exact.
    assert!(metal::log(8.0, 2.0).eq(&3.0));
    assert!(metal::log(0.25, 2.0).eq(&-2.0));
    assert!(metal::log(2.0, 4.0).eq(&0.5));

    // Both `metal::log` and `std`'s `log` are faithfully rounded, so allow a small
    // ulp distance; skip non-normal inputs and near-1 inputs (result ≈ 0).
    for i in (0..f64::INFINITY.to_bits()).step_by((1 << 46) + 1) {
        let x = f64::from_bits(i);
        if !x.is_normal() {
            continue;
        }
        for base in [2.0_f64, 3.0, 7.5, 10.0] {
            let got = metal::log(x, base);
            let want = x.log(base);
            if want.abs() < 1e-6 {
                continue;
            }
            let ulps = (got.to_bits() as i64 - want.to_bits() as i64).abs();
            assert!(
                ulps <= 4,
                "log({x:e}, {base}) = {got:e} vs std {want:e} ({ulps} ulps)"
            );
        }
    }
}
