mod bench;

bench!(bench_metallic, metallic::f32::tan, _);
bench!(bench_core_math, core_math::tanf, _);
bench!(bench_std, f32::tan, _);
bench!(bench_libm, libm::tanf, _);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
