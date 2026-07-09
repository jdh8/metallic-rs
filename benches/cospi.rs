mod bench;
// Value-uniform over a few periods, mirroring benches/sinpi.rs.  No
// `std`/`libm` legs: neither ships a `cospi`.
bench!(bench_metallic, metallic::cospi, -4.0..=4.0);
bench!(bench_core_math, core_math::cospi, -4.0..=4.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
