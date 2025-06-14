mod bench;

bench!(bench_metallic, metallic::f32::sin);
bench!(bench_core_math, core_math::sinf);
bench!(bench_std, f32::sin);
bench!(bench_libm, libm::sinf);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
