mod bench;

bench!(bench_metallic, metallic::f32::tgamma, in -10.0..=35.0);
bench!(bench_core_math, core_math::tgammaf, in -10.0..=35.0);
bench!(bench_libm, libm::tgammaf, in -10.0..=35.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_libm,);

criterion::criterion_main!(benches);
