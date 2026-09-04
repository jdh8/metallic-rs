mod bench;

bench!(bench_metallic, metallic::erff, -6.0f32..=6.0);
bench!(bench_core_math, core_math::erff, -6.0f32..=6.0);
bench!(bench_libm, libm::erff, -6.0f32..=6.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_libm);
criterion::criterion_main!(benches);
