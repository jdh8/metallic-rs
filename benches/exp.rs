mod bench;

bench!(bench_metallic, metallic::exp, -745.0..=709.0);
bench!(bench_core_math, core_math::exp, -745.0..=709.0);
bench!(bench_std, f64::exp, -745.0..=709.0);
bench!(bench_libm, libm::exp, -745.0..=709.0);

// Per-band splits at the fast leg's `q ≥ −1021` guard (issue #10): `_norm` is
// entirely above it, `_sub` entirely below (every result there is subnormal),
// so the pair prices the forced accurate-leg fallback separately from the
// fast leg.  Both arms produce subnormals in `_sub`, so any denormal-output
// assist is common-mode.

bench!(
    bench_metallic_norm,
    "metallic::exp_norm",
    metallic::exp,
    -707.0..=709.0
);
bench!(
    bench_core_math_norm,
    "core_math::exp_norm",
    core_math::exp,
    -707.0..=709.0
);
bench!(bench_std_norm, "f64::exp_norm", f64::exp, -707.0..=709.0);
bench!(bench_libm_norm, "libm::exp_norm", libm::exp, -707.0..=709.0);
bench!(
    bench_metallic_sub,
    "metallic::exp_sub",
    metallic::exp,
    -745.0..=-708.0
);
bench!(
    bench_core_math_sub,
    "core_math::exp_sub",
    core_math::exp,
    -745.0..=-708.0
);
bench!(bench_std_sub, "f64::exp_sub", f64::exp, -745.0..=-708.0);
bench!(bench_libm_sub, "libm::exp_sub", libm::exp, -745.0..=-708.0);

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
