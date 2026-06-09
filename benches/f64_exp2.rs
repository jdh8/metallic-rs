mod bench;

bench!(bench_metallic, metallic::exp2, -1075.0..=1024.0);
bench!(bench_core_math, core_math::exp2, -1075.0..=1024.0);
bench!(bench_std, f64::exp2, -1075.0..=1024.0);
bench!(bench_libm, libm::exp2, -1075.0..=1024.0);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
