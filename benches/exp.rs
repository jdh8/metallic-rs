mod bench;

bench!(bench_metallic, metallic::exp, -745.0..=709.0);
bench!(bench_core_math, core_math::exp, -745.0..=709.0);
bench!(bench_std, f64::exp, -745.0..=709.0);
bench!(bench_libm, libm::exp, -745.0..=709.0);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
