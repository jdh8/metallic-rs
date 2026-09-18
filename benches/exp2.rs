mod bench;

bench!(bench_metallic, metallic::exp2, -1075.0..=1024.0);
bench!(bench_core_math, core_math::exp2, -1075.0..=1024.0);
bench!(bench_std, f64::exp2, -1075.0..=1024.0);
bench!(bench_libm, libm::exp2, -1075.0..=1024.0);

// Per-band splits at the fast leg's `q ≥ −1021` guard (issue #10): `_norm` is
// entirely above it, `_sub` entirely below (every result there is subnormal),
// so the pair prices the forced accurate-leg fallback separately from the
// fast leg.  Both arms produce subnormals in `_sub`, so any denormal-output
// assist is common-mode.

bench!(
    bench_metallic_norm,
    "metallic::exp2_norm",
    metallic::exp2,
    -1021.0..=1024.0
);
bench!(
    bench_core_math_norm,
    "core_math::exp2_norm",
    core_math::exp2,
    -1021.0..=1024.0
);
bench!(
    bench_std_norm,
    "f64::exp2_norm",
    f64::exp2,
    -1021.0..=1024.0
);
bench!(
    bench_libm_norm,
    "libm::exp2_norm",
    libm::exp2,
    -1021.0..=1024.0
);
bench!(
    bench_metallic_sub,
    "metallic::exp2_sub",
    metallic::exp2,
    -1075.0..=-1022.0
);
bench!(
    bench_core_math_sub,
    "core_math::exp2_sub",
    core_math::exp2,
    -1075.0..=-1022.0
);
bench!(bench_std_sub, "f64::exp2_sub", f64::exp2, -1075.0..=-1022.0);
bench!(
    bench_libm_sub,
    "libm::exp2_sub",
    libm::exp2,
    -1075.0..=-1022.0
);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
    bench_metallic_norm,
    bench_core_math_norm,
    bench_std_norm,
    bench_libm_norm,
    bench_metallic_sub,
    bench_core_math_sub,
    bench_std_sub,
    bench_libm_sub,
);

criterion::criterion_main!(benches);
