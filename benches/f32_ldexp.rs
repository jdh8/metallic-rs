mod bench;

bench!(bench_metallic, metallic::f32::ldexp, .., -300..300);
bench!(bench_libm, libm::ldexpf, .., -300..300);

criterion::criterion_group!(benches, bench_metallic, bench_libm);
criterion::criterion_main!(benches);
