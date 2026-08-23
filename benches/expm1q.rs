#![feature(f128)]

mod bench;
mod bench128;

// The whole non-saturating range: below 2^-114 the result is the argument, and
// 2^15 overflows above and saturates at -1 below.
bench!(
    bench_metallic,
    metallic::expm1q,
    bench::Exponents(-114..=13)
);
bench!(
    bench_core_math,
    core_math::expm1q,
    bench::Exponents(-114..=13)
);
bench!(bench_std, f128::exp_m1, bench::Exponents(-114..=13));

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_std);
criterion::criterion_main!(benches);
