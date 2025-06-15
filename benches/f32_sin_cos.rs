mod bench;

bench!(bench_metallic, metallic::f32::sin_cos, _);
bench!(bench_core_math, core_math::sincosf, _);
bench!(bench_std, f32::sin_cos, _);
bench!(bench_libm, libm::sincosf, _);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
