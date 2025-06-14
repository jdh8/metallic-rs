mod bench;

bench!(bench_metallic, metallic::f32::ldexp; rand::random(), rand::random_range(-300..300));
bench!(bench_libm, libm::ldexpf; rand::random(), rand::random_range(-300..300));

criterion::criterion_group!(benches, bench_metallic, bench_libm);
criterion::criterion_main!(benches);
