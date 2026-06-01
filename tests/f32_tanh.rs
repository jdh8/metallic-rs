mod common;
use metallic::f32 as metal;

#[test]
fn test_tanh() {
    common::test_all_f32(metal::tanh, core_math::tanhf);
}
