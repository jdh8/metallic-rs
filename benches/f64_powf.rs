mod bench;

bench!(bench_metallic, metallic::f64::powf, .., ..);
bench!(bench_core_math, core_math::pow, .., ..);
bench!(bench_std, f64::powf, .., ..);
bench!(bench_libm, libm::pow, .., ..);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
