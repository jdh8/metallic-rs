mod bench;

bench!(bench_metallic, metallic::f32::round, ..);
bench!(bench_std, f32::round, ..);
bench!(bench_libm, libm::roundf, ..);

criterion::criterion_group!(benches, bench_metallic, bench_std, bench_libm);
criterion::criterion_main!(benches);
