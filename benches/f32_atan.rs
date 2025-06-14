mod bench;

bench!(bench_metallic, metallic::f32::atan);
bench!(bench_core_math, core_math::atanf);
bench!(bench_std, f32::atan);
bench!(bench_libm, libm::atanf);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
