#![feature(f128)]
mod bench;
mod bench128;

// CORE-MATH has no binary128 binding. libquadmath and std are timing
// context with a weaker accuracy contract, never correctness oracles.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn coshq(x: f128) -> f128;
}
bench!(bench_metallic, metallic::coshq, bench::Exponents(-20..=13));
bench!(
    bench_quadmath,
    "quadmath::coshq",
    |x| unsafe { coshq(x) },
    bench::Exponents(-20..=13)
);
bench!(bench_std, f128::cosh, bench::Exponents(-20..=13));
criterion::criterion_group!(benches, bench_metallic, bench_quadmath, bench_std);
criterion::criterion_main!(benches);
