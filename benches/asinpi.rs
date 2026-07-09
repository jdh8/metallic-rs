mod bench;
// Value-uniform over the domain: both the direct and reflection branches run.
// No `std`/`libm` legs: neither ships an `asinpi`.
bench!(bench_metallic, metallic::asinpi, -1.0..=1.0);
bench!(bench_core_math, core_math::asinpi, -1.0..=1.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
