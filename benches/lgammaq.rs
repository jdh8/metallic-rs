#![feature(f128)]
mod bench;
mod bench128;

// CORE-MATH has no binary128 binding. libquadmath and std provide timing
// context with weaker accuracy contracts; MPFR supplies the correctness oracle.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn lgammaq(x: f128) -> f128;
}
bench!(bench_metallic, metallic::lgammaq, bench::Exponents(-8..=10));
bench!(
    bench_quadmath,
    "quadmath::lgammaq",
    |x| unsafe { lgammaq(x) },
    bench::Exponents(-8..=10)
);
bench!(
    bench_std,
    "f128::ln_gamma",
    |x: f128| x.ln_gamma().0,
    bench::Exponents(-8..=10)
);
criterion::criterion_group!(benches, bench_metallic, bench_quadmath, bench_std);
criterion::criterion_main!(benches);
