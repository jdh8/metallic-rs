#![feature(f128)]

mod bench;
mod bench128;

// Neither CORE-MATH nor libquadmath binds this function. Keep a standalone
// timing measurement until a matching external implementation is available.
bench!(bench_metallic, metallic::tanpiq, bench::Exponents(-20..=20));

criterion::criterion_group!(benches, bench_metallic);
criterion::criterion_main!(benches);
