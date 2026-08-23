#![feature(f128)]

mod bench;
mod bench128;

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

// std has no `f128::exp10`, so this bench has no third lane.
criterion::criterion_group!(benches, bench_metallic, bench_core_math);
criterion::criterion_main!(benches);
