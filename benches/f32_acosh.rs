mod bench;

bench!(bench_metallic, metallic::f32::acosh; rand::random::<f32>().abs());
bench!(bench_core_math, core_math::acoshf; rand::random::<f32>().abs());
bench!(bench_std, f32::acosh; rand::random::<f32>().abs());
bench!(bench_libm, libm::acoshf; rand::random::<f32>().abs());

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
