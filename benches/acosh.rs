mod bench;

bench!(bench_metallic, metallic::acosh, 1.0..);
bench!(bench_core_math, core_math::acosh, 1.0..);
bench!(bench_std, f64::acosh, 1.0..);
bench!(bench_libm, libm::acosh, 1.0..);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
