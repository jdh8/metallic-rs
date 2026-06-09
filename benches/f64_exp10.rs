mod bench;

bench!(bench_metallic, metallic::exp10, -323.0..=308.0);
bench!(bench_core_math, core_math::exp10, -323.0..=308.0);
bench!(bench_libm, libm::exp10, -323.0..=308.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_libm);
criterion::criterion_main!(benches);
