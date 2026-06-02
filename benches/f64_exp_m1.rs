mod bench;

bench!(bench_metallic, metallic::f64::exp_m1, -40.0..=709.0);
bench!(bench_core_math, core_math::expm1, -40.0..=709.0);
bench!(bench_std, f64::exp_m1, -40.0..=709.0);
bench!(bench_libm, libm::expm1, -40.0..=709.0);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
