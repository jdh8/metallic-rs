mod bench;
// Log-uniform over `|x| ∈ [2⁻⁵⁰, 1)`, both signs — the same kernel-active band
// as the `log1p` bench (all three share the small/wide/table legs), keeping the
// deep tier's sub-2⁻⁹⁰⁰ region and the `x > 1` tail out of the measurement.
bench!(
    bench_metallic,
    metallic::log10p1,
    bench::Exponents(-50..=-1)
);
bench!(
    bench_core_math,
    core_math::log10p1,
    bench::Exponents(-50..=-1)
);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
