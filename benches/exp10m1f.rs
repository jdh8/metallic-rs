mod bench;
// Value-uniform over the two-sided kernel band, mirroring benches/exp10m1.rs.
// No `std`/`libm` legs: neither ships an `exp10m1f`.
bench!(bench_metallic, metallic::exp10m1f, -6.0f32..=6.0);
bench!(bench_core_math, core_math::exp10m1f, -6.0f32..=6.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
