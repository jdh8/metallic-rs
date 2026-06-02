mod bench;

bench!(bench_metallic, metallic::f64::hypot, .., ..);
bench!(bench_core_math, core_math::hypot, .., ..);
bench!(bench_std, f64::hypot, .., ..);
bench!(bench_libm, libm::hypot, .., ..);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
