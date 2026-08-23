#![feature(f128)]

mod bench;
mod bench128;

// The whole non-saturating range: below 2^-120 every result rounds to 1, and
// 2^15 overflows or underflows all three functions.
bench!(bench_metallic, metallic::exp2q, bench::Exponents(-120..=13));
bench!(
    bench_core_math,
    core_math::exp2q,
    bench::Exponents(-120..=13)
);
bench!(bench_std, f128::exp2, bench::Exponents(-120..=13));

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_std);
criterion::criterion_main!(benches);
