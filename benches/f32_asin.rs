mod bench;

bench!(bench_metallic, metallic::f32::asin, in -1.1..=1.1);
bench!(bench_core_math, core_math::asinf, in -1.1..=1.1);
bench!(bench_std, f32::asin, in -1.1..=1.1);
bench!(bench_libm, libm::asinf, in -1.1..=1.1);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
