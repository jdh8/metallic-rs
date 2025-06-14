mod bench;

bench!(bench_metallic, metallic::f32::ln);
bench!(bench_core_math, core_math::logf);
bench!(bench_std, f32::ln);
bench!(bench_libm, libm::logf);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
