mod bench;
// Value-uniform over the two-sided kernel band, mirroring benches/exp2m1.rs.
// No `std`/`libm` legs: neither ships an `exp2m1f`.
bench!(bench_metallic, metallic::exp2m1f, -20.0f32..=20.0);
bench!(bench_core_math, core_math::exp2m1f, -20.0f32..=20.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
