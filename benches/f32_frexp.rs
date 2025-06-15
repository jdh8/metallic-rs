mod bench;

bench!(bench_metallic, metallic::f32::frexp, _);
bench!(bench_libm, libm::frexpf, _);

criterion::criterion_group!(benches, bench_metallic, bench_libm);
criterion::criterion_main!(benches);
