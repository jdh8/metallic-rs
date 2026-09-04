#![feature(f128)]

mod bench;
mod bench128;

// Same reasoning as `benches/hypot.rs`: under representation-uniform draws the
// two magnitudes are almost always far apart, so the `dn > 56` early-out returns
// the larger leg and the kernel never runs.  `-12..=12` keeps both legs in play
// and stays clear of overflow in `x² + y²`.
bench!(
    bench_metallic,
    metallic::hypotq,
    bench::Exponents(-12..=12),
    bench::Exponents(-12..=12)
);
bench!(
    bench_core_math,
    core_math::hypotq,
    bench::Exponents(-12..=12),
    bench::Exponents(-12..=12)
);
bench!(
    bench_std,
    f128::hypot,
    bench::Exponents(-12..=12),
    bench::Exponents(-12..=12)
);

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_std);
criterion::criterion_main!(benches);
