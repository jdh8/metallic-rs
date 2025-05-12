#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

mod common;
use common::Identity as _;

/// Check if `result` is within the nearby `f32` representations of `expected`
///
/// Due to [the Table Maker's Dilemma][dilemma], it is infeasible to implement a
/// correctly-rounded (error < 0.5 ulp) transcendental function.  However,
/// faithful rounding (error < 1 ulp) is usually achievable.
///
/// [dilemma]: https://hal-lara.archives-ouvertes.fr/hal-02101765/document
///
/// If `expected` has an exact `f32` representation, `result` must be that
/// value.  Otherwise, `expected` has two `f32` neighbors, and `result` must be
/// either of them.
fn is_faithful_rounding(result: f32, expected: f64) -> bool {
    #[allow(clippy::cast_possible_truncation)]
    if result.is(&(expected as f32)) {
        return true;
    }

    let next_up: f64 = result.next_up().into();
    let next_down: f64 = result.next_down().into();
    next_down < expected && expected < next_up
}

fn test_bivariate_faithful(f: impl Fn(f32, f32) -> f32, g: impl Fn(f64, f64) -> f64) {
    common::truncate_errors((0..=u32::MAX).filter_map(|bits| {
        let x = f32::from_bits(0x10001 * (bits >> 16));
        let y = f32::from_bits(bits << 16);
        let f = f(x, y);
        let g = g(x.into(), y.into());

        (!is_faithful_rounding(f, g)).then(|| println!("{x:e}, {y:e}: {f:e} != {g:e}"))
    }));
}

fn precise_log(x: f64, base: f64) -> f64 {
    let y;
    unsafe {
        core::arch::asm!(
            "vmovsd QWORD PTR [rsp - 16], {x}",
            "vmovsd QWORD PTR [rsp - 8], {base}",
            "fld1",
            "fld    QWORD PTR [rsp - 8]",
            "fyl2x",
            "fld1",
            "fld    QWORD PTR [rsp - 16]",
            "fyl2x",
            "fdivrp",
            "fstp   QWORD PTR [rsp - 16]",
            "vmovsd {y}, QWORD PTR [rsp - 16]",
            x = in(xmm_reg) x,
            base = in(xmm_reg) base,
            y = out(xmm_reg) y,
            options(pure, nomem),
        );
    }
    y
}

#[test]
fn test_precise_log() {
    assert!(precise_log(125.0, 5.0).eq(&3.0));
}

#[test]
fn test_log() {
    test_bivariate_faithful(metallic::f32::log, precise_log);
}
