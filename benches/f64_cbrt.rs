mod bench;

bench!(bench_metallic, metallic::f64::cbrt, _);
bench!(bench_core_math, core_math::cbrt, _);
bench!(bench_std, f64::cbrt, _);
bench!(bench_libm, libm::cbrt, _);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
