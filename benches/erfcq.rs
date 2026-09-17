#![feature(f128)]
mod bench;
mod bench128;

// CORE-MATH has no binary128 binding. libquadmath and std provide timing
// context with weaker accuracy contracts; MPFR supplies the correctness oracle.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn erfcq(x: f128) -> f128;
}
bench!(bench_metallic, metallic::erfcq, bench::Exponents(-8..=6));
bench!(
    bench_quadmath,
    "quadmath::erfcq",
    |x| unsafe { erfcq(x) },
    bench::Exponents(-8..=6)
);
bench!(bench_std, f128::erfc, bench::Exponents(-8..=6));
criterion::criterion_group!(benches, bench_metallic, bench_quadmath, bench_std);
criterion::criterion_main!(benches);
