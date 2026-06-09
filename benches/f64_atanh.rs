mod bench;

bench!(bench_metallic, metallic::atanh, -1.05..=1.05);
bench!(bench_core_math, core_math::atanh, -1.05..=1.05);
bench!(bench_std, f64::atanh, -1.05..=1.05);
bench!(bench_libm, libm::atanh, -1.05..=1.05);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
