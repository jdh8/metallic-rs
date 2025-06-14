mod bench;

bench!(bench_metallic, metallic::f32::powf; rand::random(), rand::random());
bench!(bench_core_math, core_math::powf; rand::random(), rand::random());
bench!(bench_std, f32::powf; rand::random(), rand::random());
bench!(bench_libm, libm::powf; rand::random(), rand::random());

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
