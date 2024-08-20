fn precise_log(x: f32, base: f32) -> f32 {
    let y;
    unsafe {
        core::arch::asm!(
            "vmovss DWORD PTR [rsp - 8], {x}",
            "vmovss DWORD PTR [rsp - 4], {base}",
            "fld1",
            "fld    DWORD PTR [rsp - 4]",
            "fyl2x",
            "fld1",
            "fld    DWORD PTR [rsp - 8]",
            "fyl2x",
            "fdivrp",
            "fstp   DWORD PTR [rsp - 8]",
            "vmovss {y}, DWORD PTR [rsp - 8]",
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
    super::test_bivariate_usual(metallic::f32::log, precise_log);
}
