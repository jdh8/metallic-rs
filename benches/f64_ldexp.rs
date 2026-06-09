mod bench;

bench!(bench_metallic, metallic::ldexp, .., -2200..2200);
bench!(bench_libm, libm::ldexp, .., -2200..2200);

criterion::criterion_group!(benches, bench_metallic, bench_libm);
criterion::criterion_main!(benches);
