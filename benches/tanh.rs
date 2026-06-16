mod bench;
// Log-uniform over the kernel band `[⅛, 20)`.  Representation-uniform `..` would
// land ~99.6% of draws in the small-series leg (`|x| < ⅛`) or the `≥ 20`
// saturation, almost never timing the poly/table kernel.  `-3..=3` = `[0.125, 16)`
// keeps every draw in the kernel (signed; tanh is odd).  See `bench::Exponents`.
bench!(bench_metallic, metallic::tanh, bench::Exponents(-3..=3));
bench!(bench_core_math, core_math::tanh, bench::Exponents(-3..=3));
bench!(bench_std, f64::tanh, bench::Exponents(-3..=3));
bench!(bench_libm, libm::tanh, bench::Exponents(-3..=3));

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
