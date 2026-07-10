mod bench;
// Representation-uniform over the domain `(-1, ∞)`, following log1pf's band.
// No `std`/`libm` legs: neither ships a `log2p1f`.
bench!(bench_metallic, metallic::log2p1f, -1.0..);
bench!(bench_core_math, core_math::log2p1f, -1.0..);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
