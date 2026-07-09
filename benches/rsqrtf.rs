mod bench;
// Representation-uniform over `[+0, +∞]`: the promoted-f64 kernel runs for
// every positive input, so no magnitude band is needed.  No `std`/`libm` legs:
// neither ships an `rsqrtf`.
bench!(bench_metallic, metallic::rsqrtf, 0.0f32..);
bench!(bench_core_math, core_math::rsqrtf, 0.0f32..);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
