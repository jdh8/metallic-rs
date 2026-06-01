mod bench;

bench!(bench_metallic, metallic::f32::hypot, .., ..);
bench!(bench_core_math, core_math::hypotf, .., ..);
bench!(bench_std, f32::hypot, .., ..);
bench!(bench_libm, libm::hypotf, .., ..);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
