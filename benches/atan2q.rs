#![feature(f128)]

mod bench;
mod bench128;

// Same reasoning as `benches/atan2.rs`: log-uniform over both args keeps the
// ratio's exponent in the kernel band, so both sides stay on their fast path
// and the specials never fire.  The band still exercises every quadrant, the
// swap, and the sector tables.
bench!(
    bench_metallic,
    metallic::atan2q,
    bench::Exponents(-20..=20),
    bench::Exponents(-20..=20)
);
bench!(
    bench_core_math,
    core_math::atan2q,
    bench::Exponents(-20..=20),
    bench::Exponents(-20..=20)
);
bench!(
    bench_std,
    f128::atan2,
    bench::Exponents(-20..=20),
    bench::Exponents(-20..=20)
);

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_std);
criterion::criterion_main!(benches);
