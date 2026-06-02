mod bench;

bench!(bench_metallic, metallic::f64::sinh, -710.0..=710.0);
bench!(bench_core_math, core_math::sinh, -710.0..=710.0);
bench!(bench_std, f64::sinh, -710.0..=710.0);
bench!(bench_libm, libm::sinh, -710.0..=710.0);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
