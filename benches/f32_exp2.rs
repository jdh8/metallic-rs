mod bench;

bench!(bench_metallic, metallic::f32::exp2, in -155.0..=130.0);
bench!(bench_core_math, core_math::exp2f, in -155.0..=130.0);
bench!(bench_std, f32::exp2, in -155.0..=130.0);
bench!(bench_libm, libm::exp2f, in -155.0..=130.0);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
