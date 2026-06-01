mod bench;

bench!(bench_metallic, metallic::f32::cosh, -90.0..=90.0);
bench!(bench_core_math, core_math::coshf, -90.0..=90.0);
bench!(bench_std, f32::cosh, -90.0..=90.0);
bench!(bench_libm, libm::coshf, -90.0..=90.0);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
