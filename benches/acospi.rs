mod bench;
// Value-uniform over the domain.  No `std`/`libm` legs: neither ships an
// `acospi`.
bench!(bench_metallic, metallic::acospi, -1.0..=1.0);
bench!(bench_core_math, core_math::acospi, -1.0..=1.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
