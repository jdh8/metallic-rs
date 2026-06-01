mod bench;

bench!(bench_metallic, metallic::f32::tan, ..);
bench!(bench_core_math, core_math::tanf, ..);
bench!(bench_std, f32::tan, ..);
bench!(bench_libm, libm::tanf, ..);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
