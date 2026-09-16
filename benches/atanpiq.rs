#![feature(f128)]

mod bench;
mod bench128;

// No external binary128 binding yet; use atanq's active band.
bench!(
    bench_metallic,
    metallic::atanpiq,
    bench::Exponents(-20..=20)
);

criterion::criterion_group!(benches, bench_metallic);
criterion::criterion_main!(benches);
