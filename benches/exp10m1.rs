mod bench;
// Value-uniform over the two-sided kernel band: past -16.3 everything is the
// constant -1 and past 308 it is infinity, so an open-ended draw would
// measure fast returns.  No `std`/`libm` legs: neither ships an `exp10m1`.
bench!(bench_metallic, metallic::exp10m1, -6.0..=6.0);
bench!(bench_core_math, core_math::exp10m1, -6.0..=6.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
