#![feature(f128)]
mod bench;
mod bench128;

// CORE-MATH has no binary128 binding. libquadmath and std are timing
// context with a weaker accuracy contract, never correctness oracles.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn acoshq(x: f128) -> f128;
}
bench!(
    bench_metallic,
    metallic::acoshq,
    bench::PositiveExponents(0..=10)
);
bench!(
    bench_quadmath,
    "quadmath::acoshq",
    |x| unsafe { acoshq(x) },
    bench::PositiveExponents(0..=10)
);
bench!(bench_std, f128::acosh, bench::PositiveExponents(0..=10));
criterion::criterion_group!(benches, bench_metallic, bench_quadmath, bench_std);
criterion::criterion_main!(benches);
