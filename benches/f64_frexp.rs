mod bench;

bench!(bench_metallic, metallic::f64::frexp, ..);
bench!(bench_libm, libm::frexp, ..);

criterion::criterion_group!(benches, bench_metallic, bench_libm);
criterion::criterion_main!(benches);
