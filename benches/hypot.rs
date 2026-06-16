mod bench;
// Log-uniform over both args with `|Δexp| ≤ 24 < 27`, so neither arg is negligible
// and the `√(x²+y²)` kernel always runs.  Under representation-uniform `.., ..`
// the two magnitudes are almost always far apart → the smaller is dropped and the
// fast `|larger|` path dominates (plus inf/NaN).  `-12..=12` also stays clear of
// overflow/underflow in `x²+y²`.  See `bench::Exponents`.
bench!(
    bench_metallic,
    metallic::hypot,
    bench::Exponents(-12..=12),
    bench::Exponents(-12..=12)
);
bench!(
    bench_core_math,
    core_math::hypot,
    bench::Exponents(-12..=12),
    bench::Exponents(-12..=12)
);
bench!(
    bench_std,
    f64::hypot,
    bench::Exponents(-12..=12),
    bench::Exponents(-12..=12)
);
bench!(
    bench_libm,
    libm::hypot,
    bench::Exponents(-12..=12),
    bench::Exponents(-12..=12)
);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
