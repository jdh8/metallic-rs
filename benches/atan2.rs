mod bench;
bench!(bench_metallic, metallic::atan2, .., ..);
bench!(bench_core_math, core_math::atan2, .., ..);
bench!(bench_std, f64::atan2, .., ..);
bench!(bench_libm, libm::atan2, .., ..);
criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm
);
criterion::criterion_main!(benches);
