mod bench;

bench!(bench_metallic, metallic::f64::log, 0.0.., 0.0..);
bench!(bench_std, f64::log, 0.0.., 0.0..);

criterion::criterion_group!(benches, bench_metallic, bench_std);
criterion::criterion_main!(benches);
