mod bench;

// Cover the nonsaturated region through the binary32 underflow boundary.
bench!(bench_metallic, metallic::erfcf, -6.0f32..=10.0);
bench!(bench_core_math, core_math::erfcf, -6.0f32..=10.0);
bench!(bench_libm, libm::erfcf, -6.0f32..=10.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_libm);
criterion::criterion_main!(benches);
