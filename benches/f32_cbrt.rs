mod bench;

bench!(bench_metallic, metallic::f32::cbrt);
bench!(bench_core_math, core_math::cbrtf);
bench!(bench_std, f32::cbrt);
bench!(bench_libm, libm::cbrtf);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
