mod bench;

// Moderate finite operands exercise fused arithmetic without overflow or
// underflow. CORE-MATH has no fmaf entry point; libm supplies the comparison.
bench!(
    bench_metallic,
    metallic::fmaf,
    -16.0f32..=16.0,
    -16.0f32..=16.0,
    -16.0f32..=16.0
);
bench!(
    bench_libm,
    libm::fmaf,
    -16.0f32..=16.0,
    -16.0f32..=16.0,
    -16.0f32..=16.0
);

criterion::criterion_group!(benches, bench_metallic, bench_libm);
criterion::criterion_main!(benches);
