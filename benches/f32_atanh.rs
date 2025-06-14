mod bench;

bench!(bench_metallic, metallic::f32::atanh, in -1.05..1.05);
bench!(bench_core_math, core_math::atanhf, in -1.05..1.05);
bench!(bench_std, f32::atanh, in -1.05..1.05);
bench!(bench_libm, libm::atanhf, in -1.05..1.05);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
