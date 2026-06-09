mod bench;

bench!(bench_metallic, metallic::erfc, -6.0..=28.0);
bench!(bench_core_math, core_math::erfc, -6.0..=28.0);
bench!(bench_libm, libm::erfc, -6.0..=28.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_libm,);

criterion::criterion_main!(benches);
