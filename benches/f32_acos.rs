mod bench;

bench!(bench_metallic, metallic::acosf, -1.1..=1.1);
bench!(bench_core_math, core_math::acosf, -1.1..=1.1);
bench!(bench_std, f32::acos, -1.1..=1.1);
bench!(bench_libm, libm::acosf, -1.1..=1.1);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
