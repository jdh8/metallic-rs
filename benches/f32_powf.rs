mod bench;

bench!(bench_metallic, metallic::f32::powf, .., ..);
bench!(bench_core_math, core_math::powf, .., ..);
bench!(bench_std, f32::powf, .., ..);
bench!(bench_libm, libm::powf, .., ..);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
