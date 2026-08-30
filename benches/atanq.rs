#![feature(f128)]

mod bench;
mod bench128;

// Same band as `benches/atan2q.rs`: log-uniform exponents keep the argument in
// the kernel band where the reduction works hardest, while still exercising
// every sector, both sides of the swap, and the tiny/huge fast-outs.
//
// The published `core-math` crate does not bind `atanq` yet, so the baseline
// is `atan2q(x, 1)` — the same correctly rounded value through the same
// upstream pipeline, but with the bivariate plumbing it cannot constant-fold
// across the FFI boundary.  Read the ratio as a lower bound and re-baseline
// on upstream's dedicated `atanq` once the next `core-math` release lands.
bench!(bench_metallic, metallic::atanq, bench::Exponents(-20..=20));
bench!(
    bench_core_math,
    |x| core_math::atan2q(x, 1.0),
    bench::Exponents(-20..=20)
);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);
criterion::criterion_main!(benches);
