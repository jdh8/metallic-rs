mod bench;

bench!(bench_metallic, metallic::f64::sin, ..);
bench!(bench_core_math, core_math::sin, ..);
bench!(bench_std, f64::sin, ..);
bench!(bench_libm, libm::sin, ..);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm
);
criterion::criterion_main!(benches);
