mod bench;
// Independent log-uniform magnitudes on both arguments (mirrors
// benches/atan2.rs).  No `std`/`libm` legs: neither ships an `atan2pi`.
bench!(
    bench_metallic,
    metallic::atan2pi,
    bench::Exponents(-20..=20),
    bench::Exponents(-20..=20)
);
bench!(
    bench_core_math,
    core_math::atan2pi,
    bench::Exponents(-20..=20),
    bench::Exponents(-20..=20)
);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
