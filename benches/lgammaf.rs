mod bench;

bench!(bench_metallic, metallic::lgammaf, -10.0..=35.0);
bench!(bench_core_math, core_math::lgammaf, -10.0..=35.0);
bench!(bench_libm, libm::lgammaf, -10.0..=35.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_libm,);

criterion::criterion_main!(benches);
