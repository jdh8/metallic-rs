#![feature(f128)]

mod bench;
mod bench128;

// Neither CORE-MATH nor libquadmath binds this entry point. The main band
// matches expm1q; the separate tiny band measures the irrational slope.
bench!(
    bench_metallic,
    metallic::exp10m1q,
    bench::Exponents(-114..=13)
);

fn exp10m1q_tiny(x: f128) -> f128 {
    metallic::exp10m1q(x)
}
bench!(bench_tiny, exp10m1q_tiny, bench::Exponents(-16382..=-115));

criterion::criterion_group!(benches, bench_metallic, bench_tiny);
criterion::criterion_main!(benches);
