mod bench;
// Log-uniform over `|x| ∈ [2⁻⁵⁰, 1)`, both signs.  Representation-uniform `-1.0..`
// puts ~38% of draws in the `|x| < 2⁻⁵⁴` fast return.  `-50..=-1` excludes that
// fast return and the `x > 1` region (already covered by the `log` bench), and
// keeps the negative tail above `-1` (in domain), sampling the small-`x` and
// near-`(-1)` cancellation kernel.  See `bench::Exponents`.
bench!(bench_metallic, metallic::log1p, bench::Exponents(-50..=-1));
bench!(
    bench_core_math,
    core_math::log1p,
    bench::Exponents(-50..=-1)
);
bench!(bench_std, f64::ln_1p, bench::Exponents(-50..=-1));
bench!(bench_libm, libm::log1p, bench::Exponents(-50..=-1));

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
