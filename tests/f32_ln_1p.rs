mod common;
use metallic::f32 as metal;

#[test]
fn test_ln_1p() {
    common::test_all_f32(metal::ln_1p, core_math::log1pf);
}
