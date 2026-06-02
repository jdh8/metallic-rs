mod bench;

bench!(bench_metallic, metallic::f64::sin_cos, ..);
bench!(bench_std, f64::sin_cos, ..);
bench!(bench_libm, libm::sincos, ..);

criterion::criterion_group!(benches, bench_metallic, bench_std, bench_libm);
criterion::criterion_main!(benches);
