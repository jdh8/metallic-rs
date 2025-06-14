mod bench;

bench!(bench_metallic, metallic::f32::exp_m1, in -20.0..90.0);
bench!(bench_core_math, core_math::expm1f, in -20.0..90.0);
bench!(bench_std, f32::exp_m1, in -20.0..90.0);
bench!(bench_libm, libm::expm1f, in -20.0..90.0);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
