mod bench;

bench!(bench_metallic, metallic::f64::round, ..);
bench!(bench_std, f64::round, ..);
bench!(bench_libm, libm::round, ..);

criterion::criterion_group!(benches, bench_metallic, bench_std, bench_libm);
criterion::criterion_main!(benches);
