mod bench;

bench!(bench_metallic, metallic::log1pf, -1.0..);
bench!(bench_core_math, core_math::log1pf, -1.0..);
bench!(bench_std, f32::ln_1p, -1.0..);
bench!(bench_libm, libm::log1pf, -1.0..);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
