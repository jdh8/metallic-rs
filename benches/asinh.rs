mod bench;
// Log-uniform over the kernel band `[2⁻⁴, 2²⁷)` — the sqrt + `ln_dd` path.
// Representation-uniform `..` would split ~half into the small-series leg
// (`|x| < 0.0625`) and the rest into the `|x| ≥ 2²⁷` sqrt-free asymptotic
// `ln(2x)` tail.  `-4..=26` keeps every draw in the kernel (signed; asinh is
// odd).  See `bench::Exponents`.
bench!(bench_metallic, metallic::asinh, bench::Exponents(-4..=26));
bench!(bench_core_math, core_math::asinh, bench::Exponents(-4..=26));
bench!(bench_std, f64::asinh, bench::Exponents(-4..=26));
bench!(bench_libm, libm::asinh, bench::Exponents(-4..=26));

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
