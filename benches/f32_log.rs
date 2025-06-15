mod bench;

bench!(bench_metallic, metallic::f32::log, _, _);
bench!(bench_std, f32::log, _, _);

criterion::criterion_group!(benches, bench_metallic, bench_std);
criterion::criterion_main!(benches);
