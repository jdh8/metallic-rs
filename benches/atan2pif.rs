mod bench;
// Representation-uniform pairs.  No `std`/`libm` legs: neither ships an
// `atan2pif`.
bench!(bench_metallic, metallic::atan2pif, .., ..);
bench!(bench_core_math, core_math::atan2pif, .., ..);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
