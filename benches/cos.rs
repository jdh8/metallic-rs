mod bench;

bench!(bench_metallic, metallic::cos, ..);
bench!(bench_core_math, core_math::cos, ..);
bench!(bench_std, f64::cos, ..);
bench!(bench_libm, libm::cos, ..);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm
);
criterion::criterion_main!(benches);
