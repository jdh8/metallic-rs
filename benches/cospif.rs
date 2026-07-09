mod bench;
// Value-uniform over a few periods, mirroring benches/sinpif.rs.  No
// `std`/`libm` legs: neither ships a `cospif`.
bench!(bench_metallic, metallic::cospif, -4.0f32..=4.0);
bench!(bench_core_math, core_math::cospif, -4.0f32..=4.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
