mod bench;

bench!(bench_metallic, metallic::f64::asinh, ..);
bench!(bench_core_math, core_math::asinh, ..);
bench!(bench_std, f64::asinh, ..);
bench!(bench_libm, libm::asinh, ..);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
