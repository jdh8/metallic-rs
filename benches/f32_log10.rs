mod bench;

bench!(bench_metallic, metallic::f32::log10);
bench!(bench_core_math, core_math::log10f);
bench!(bench_std, f32::log10);
bench!(bench_libm, libm::log10f);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
