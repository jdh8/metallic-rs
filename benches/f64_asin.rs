mod bench;
bench!(bench_metallic, metallic::asin, -1.0..=1.0);
bench!(bench_core_math, core_math::asin, -1.0..=1.0);
bench!(bench_std, f64::asin, -1.0..=1.0);
bench!(bench_libm, libm::asin, -1.0..=1.0);
criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm
);
criterion::criterion_main!(benches);
