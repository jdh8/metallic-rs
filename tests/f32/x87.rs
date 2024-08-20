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
    const F: fn(f32, f32) -> f32 = metallic::f32::log;
    const G: fn(f32, f32) -> f32 = precise_log;

    let count = (0..=u32::MAX)
        .filter_map(|bits| {
            let x = f32::from_bits(0x10001 * (bits >> 16));
            let y = f32::from_bits(bits << 16);

            (!super::is(F(x, y), G(x, y)))
                .then(|| Some(println!("{x:e}, {y:e}: {:e} != {:e}", F(x, y), G(x, y))))
        })
        .count();

    assert!(count == 0, "There are {count} mismatches");
}
