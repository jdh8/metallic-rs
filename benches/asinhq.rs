#![feature(f128)]
mod bench;
mod bench128;

// CORE-MATH has no binary128 binding. libquadmath and std are timing
// context with a weaker accuracy contract, never correctness oracles.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn asinhq(x: f128) -> f128;
}
bench!(bench_metallic, metallic::asinhq, bench::Exponents(-20..=20));
bench!(
    bench_quadmath,
    "quadmath::asinhq",
    |x| unsafe { asinhq(x) },
    bench::Exponents(-20..=20)
);
bench!(bench_std, f128::asinh, bench::Exponents(-20..=20));
criterion::criterion_group!(benches, bench_metallic, bench_quadmath, bench_std);
criterion::criterion_main!(benches);
