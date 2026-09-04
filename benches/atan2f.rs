mod bench;

// Bounded draws keep both arguments in the active range and avoid the
// negligible-ratio and nonfinite cases that dominate full-representation pairs.
bench!(
    bench_metallic,
    metallic::atan2f,
    -1.0f32..=1.0,
    -1.0f32..=1.0
);
bench!(
    bench_core_math,
    core_math::atan2f,
    -1.0f32..=1.0,
    -1.0f32..=1.0
);
bench!(bench_std, f32::atan2, -1.0f32..=1.0, -1.0f32..=1.0);
bench!(bench_libm, libm::atan2f, -1.0f32..=1.0, -1.0f32..=1.0);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm
);
criterion::criterion_main!(benches);
