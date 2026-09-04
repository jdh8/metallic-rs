#![feature(f128)]

mod bench;
mod bench128;

// Same band as `benches/atan2q.rs`: log-uniform exponents keep the argument in
// the kernel band where the reduction works hardest, while still exercising
// every sector, both sides of the swap, and the tiny/huge fast-outs.
bench!(bench_metallic, metallic::atanq, bench::Exponents(-20..=20));
bench!(
    bench_core_math,
    core_math::atanq,
    bench::Exponents(-20..=20)
);
bench!(bench_std, f128::atan, bench::Exponents(-20..=20));

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_std);
criterion::criterion_main!(benches);
