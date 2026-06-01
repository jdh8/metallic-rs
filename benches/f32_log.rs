mod bench;

bench!(bench_metallic, metallic::f32::log, in 0.0..=f32::INFINITY, in 0.0..=f32::INFINITY);
bench!(bench_std, f32::log, in 0.0..=f32::INFINITY, in 0.0..=f32::INFINITY);

criterion::criterion_group!(benches, bench_metallic, bench_std);
criterion::criterion_main!(benches);
