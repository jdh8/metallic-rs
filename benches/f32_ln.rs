mod bench;

bench!(bench_metallic, metallic::f32::ln, in 0.0..=f32::INFINITY);
bench!(bench_core_math, core_math::logf, in 0.0..=f32::INFINITY);
bench!(bench_std, f32::ln, in 0.0..=f32::INFINITY);
bench!(bench_libm, libm::logf, in 0.0..=f32::INFINITY);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
