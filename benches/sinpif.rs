mod bench;
// Value-uniform over a few periods, mirroring benches/sinpi.rs.  No
// `std`/`libm` legs: neither ships a `sinpif`.
bench!(bench_metallic, metallic::sinpif, -4.0f32..=4.0);
bench!(bench_core_math, core_math::sinpif, -4.0f32..=4.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
