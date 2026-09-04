#![feature(f128)]

mod bench;
mod bench128;

// glibc's libm does have `exp10f128`, but std binds no `f128::exp10` method to
// it, so this lane is declared and labelled by the C symbol rather than the
// `f128::` path the other benches use.
unsafe extern "C" {
    fn exp10f128(x: f128) -> f128;
}

// The whole non-saturating range: below 2^-120 every result rounds to 1, and
// 2^15 overflows or underflows all three functions.
bench!(
    bench_metallic,
    metallic::exp10q,
    bench::Exponents(-120..=13)
);
bench!(
    bench_core_math,
    core_math::exp10q,
    bench::Exponents(-120..=13)
);
bench!(
    bench_glibc,
    "glibc::exp10f128",
    |x| unsafe { exp10f128(x) },
    bench::Exponents(-120..=13)
);

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_glibc);
criterion::criterion_main!(benches);
