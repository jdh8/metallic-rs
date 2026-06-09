mod bench;

bench!(bench_metallic, metallic::erf, -6.0..=6.0);
bench!(bench_core_math, core_math::erf, -6.0..=6.0);
bench!(bench_libm, libm::erf, -6.0..=6.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_libm,);

criterion::criterion_main!(benches);
