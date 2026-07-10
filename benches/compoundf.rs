mod bench;
// Value-uniform base band around 1 and moderate exponents, keeping `2^(y·log2(1+x))`
// inside the finite range so both sides run their kernels rather than the
// overflow/underflow fast returns.  No `std`/`libm` legs: neither ships a `compoundf`.
bench!(
    bench_metallic,
    metallic::compoundf,
    -0.5f32..=2.0,
    -50.0f32..=50.0
);
bench!(
    bench_core_math,
    core_math::compoundf,
    -0.5f32..=2.0,
    -50.0f32..=50.0
);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
