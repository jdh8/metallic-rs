mod bench;
// Representation-uniform: the f32 domain has no fast-return band wide enough
// to swamp it.  No `std`/`libm` legs: neither ships an `atanpif`.
bench!(bench_metallic, metallic::atanpif, ..);
bench!(bench_core_math, core_math::atanpif, ..);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
