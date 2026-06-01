mod bench;

bench!(bench_metallic, metallic::f32::acosh, in 1.0..=f32::INFINITY);
bench!(bench_core_math, core_math::acoshf, in 1.0..=f32::INFINITY);
bench!(bench_std, f32::acosh, in 1.0..=f32::INFINITY);
bench!(bench_libm, libm::acoshf, in 1.0..=f32::INFINITY);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
