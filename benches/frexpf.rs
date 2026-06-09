mod bench;

bench!(bench_metallic, metallic::frexpf, ..);
bench!(bench_libm, libm::frexpf, ..);

criterion::criterion_group!(benches, bench_metallic, bench_libm);
criterion::criterion_main!(benches);
