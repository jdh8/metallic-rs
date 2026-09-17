#![feature(f128)]
mod bench;
mod bench128;

// CORE-MATH has no binary128 binding. libquadmath and std provide timing
// context with weaker accuracy contracts; MPFR supplies the correctness oracle.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn tgammaq(x: f128) -> f128;
}
bench!(bench_metallic, metallic::tgammaq, bench::Exponents(-8..=10));
bench!(
    bench_quadmath,
    "quadmath::tgammaq",
    |x| unsafe { tgammaq(x) },
    bench::Exponents(-8..=10)
);
bench!(bench_std, f128::gamma, bench::Exponents(-8..=10));
criterion::criterion_group!(benches, bench_metallic, bench_quadmath, bench_std);
criterion::criterion_main!(benches);
