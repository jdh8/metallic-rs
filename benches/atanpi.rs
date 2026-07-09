mod bench;
// Log-uniform over the kernel band (mirrors benches/atan.rs): fast returns at
// the extremes would swamp a representation-uniform draw.  No `std`/`libm`
// legs: neither ships an `atanpi`.
bench!(bench_metallic, metallic::atanpi, bench::Exponents(-40..=39));
bench!(
    bench_core_math,
    core_math::atanpi,
    bench::Exponents(-40..=39)
);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
