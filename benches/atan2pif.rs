mod bench;
// Match atan2f's active band: representation-uniform pairs overwhelmingly
// approach an axis and obscure the cost of the general quadrant reduction.
// No `std`/`libm` legs: neither ships an `atan2pif`.
bench!(
    bench_metallic,
    metallic::atan2pif,
    -1.0f32..=1.0,
    -1.0f32..=1.0
);
bench!(
    bench_core_math,
    core_math::atan2pif,
    -1.0f32..=1.0,
    -1.0f32..=1.0
);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
