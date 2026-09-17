#![feature(f128)]
mod bench;
mod bench128;

// CORE-MATH has no binary128 binding. libquadmath and std provide timing
// context with weaker accuracy contracts; MPFR supplies the correctness oracle.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn erfq(x: f128) -> f128;
}
bench!(bench_metallic, metallic::erfq, bench::Exponents(-20..=3));
bench!(
    bench_quadmath,
    "quadmath::erfq",
    |x| unsafe { erfq(x) },
    bench::Exponents(-20..=3)
);
bench!(bench_std, f128::erf, bench::Exponents(-20..=3));
criterion::criterion_group!(benches, bench_metallic, bench_quadmath, bench_std);
criterion::criterion_main!(benches);
