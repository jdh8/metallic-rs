mod bench;

bench!(bench_metallic, metallic::lgamma, -10.0..=200.0);
bench!(bench_core_math, core_math::lgamma, -10.0..=200.0);
bench!(bench_libm, libm::lgamma, -10.0..=200.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_libm,);

criterion::criterion_main!(benches);
