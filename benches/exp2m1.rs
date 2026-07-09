mod bench;
// Value-uniform over the two-sided kernel band: past -54 everything is the
// constant -1 and past 1024 it is infinity, so an open-ended draw would
// measure fast returns.  No `std`/`libm` legs: neither ships an `exp2m1`.
bench!(bench_metallic, metallic::exp2m1, -20.0..=20.0);
bench!(bench_core_math, core_math::exp2m1, -20.0..=20.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
