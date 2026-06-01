mod bench;

bench!(bench_metallic, metallic::f32::log2, 0.0..);
bench!(bench_core_math, core_math::log2f, 0.0..);
bench!(bench_std, f32::log2, 0.0..);
bench!(bench_libm, libm::log2f, 0.0..);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
