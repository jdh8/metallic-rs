mod bench;

bench!(bench_metallic, metallic::exp10, -323.0..=308.0);
bench!(bench_core_math, core_math::exp10, -323.0..=308.0);
bench!(bench_libm, libm::exp10, -323.0..=308.0);

// Per-band splits at the fast leg's `q ≥ −1021` guard (issue #10): `_norm` is
// entirely above it, `_sub` entirely below (every result there is subnormal),
// so the pair prices the forced accurate-leg fallback separately from the
// fast leg.  Both arms produce subnormals in `_sub`, so any denormal-output
// assist is common-mode.

bench!(
    bench_metallic_norm,
    "metallic::exp10_norm",
    metallic::exp10,
    -307.0..=308.0
);
bench!(
    bench_core_math_norm,
    "core_math::exp10_norm",
    core_math::exp10,
    -307.0..=308.0
);
bench!(
    bench_libm_norm,
    "libm::exp10_norm",
    libm::exp10,
    -307.0..=308.0
);
bench!(
    bench_metallic_sub,
    "metallic::exp10_sub",
    metallic::exp10,
    -323.0..=-308.0
);
bench!(
    bench_core_math_sub,
    "core_math::exp10_sub",
    core_math::exp10,
    -323.0..=-308.0
);
bench!(
    bench_libm_sub,
    "libm::exp10_sub",
    libm::exp10,
    -323.0..=-308.0
);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_libm,
    bench_metallic_norm,
    bench_core_math_norm,
    bench_libm_norm,
    bench_metallic_sub,
    bench_core_math_sub,
    bench_libm_sub,
);

criterion::criterion_main!(benches);
