#![feature(f128)]

mod bench;
mod bench128;

// No external binary128 binding yet; use atan2q's active ratio band.
bench!(
    bench_metallic,
    metallic::atan2piq,
    bench::Exponents(-20..=20),
    bench::Exponents(-20..=20)
);

criterion::criterion_group!(benches, bench_metallic);
criterion::criterion_main!(benches);
