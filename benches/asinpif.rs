mod bench;
// Value-uniform over the domain, mirroring benches/asinpi.rs.  No `std`/`libm`
// legs: neither ships an `asinpif`.
bench!(bench_metallic, metallic::asinpif, -1.0f32..=1.0);
bench!(bench_core_math, core_math::asinpif, -1.0f32..=1.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
