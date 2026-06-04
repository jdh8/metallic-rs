mod bench;

bench!(bench_metallic, metallic::f64::tgamma, -10.0..=35.0);
bench!(bench_libm, libm::tgamma, -10.0..=35.0);

criterion::criterion_group!(benches, bench_metallic, bench_libm,);

criterion::criterion_main!(benches);
