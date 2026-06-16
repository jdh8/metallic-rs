mod bench;
// Log-uniform over `[1, 2048)`.  Representation-uniform `1.0..` gives the near-1
// series region `[1, 2)` (the distinctive cancellation case) only ~0.1% of draws,
// the rest being the large-`x` log path.  `PositiveExponents(0..=10)` gives each
// binade — including `[1, 2)` — an equal ~1/11 share while still covering the log
// path.  See `bench::PositiveExponents`.
bench!(
    bench_metallic,
    metallic::acosh,
    bench::PositiveExponents(0..=10)
);
bench!(
    bench_core_math,
    core_math::acosh,
    bench::PositiveExponents(0..=10)
);
bench!(bench_std, f64::acosh, bench::PositiveExponents(0..=10));
bench!(bench_libm, libm::acosh, bench::PositiveExponents(0..=10));

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
