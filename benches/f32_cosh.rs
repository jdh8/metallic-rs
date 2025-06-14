mod bench;

bench!(bench_metallic, metallic::f32::cosh);
bench!(bench_core_math, core_math::coshf);
bench!(bench_std, f32::cosh);
bench!(bench_libm, libm::coshf);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
