mod bench;

bench!(bench_metallic, metallic::f32::cos);
bench!(bench_core_math, core_math::cosf);
bench!(bench_std, f32::cos);
bench!(bench_libm, libm::cosf);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
