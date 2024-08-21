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
    super::test_bivariate_faithful(metallic::f32::log, precise_log);
}
