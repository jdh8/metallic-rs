mod bench;

bench!(bench_metallic, metallic::lgammaf, -10.0..=35.0);
bench!(bench_core_math, core_math::lgammaf, -10.0..=35.0);
bench!(bench_libm, libm::lgammaf, -10.0..=35.0);

// Per-band splits mirroring benches/lgamma.rs: reflection (z < 0.5), the
// log-free rational [0.5, 8), and Stirling [8, 35].
bench!(
    bench_metallic_reflect,
    "metallic::lgammaf_reflect",
    metallic::lgammaf,
    -10.0..=-2.0
);
bench!(
    bench_core_math_reflect,
    "core_math::lgammaf_reflect",
    core_math::lgammaf,
    -10.0..=-2.0
);
bench!(
    bench_metallic_rational,
    "metallic::lgammaf_rational",
    metallic::lgammaf,
    0.5..=8.0
);
bench!(
    bench_core_math_rational,
    "core_math::lgammaf_rational",
    core_math::lgammaf,
    0.5..=8.0
);
bench!(
    bench_metallic_stirling,
    "metallic::lgammaf_stirling",
    metallic::lgammaf,
    8.0..=35.0
);
bench!(
    bench_core_math_stirling,
    "core_math::lgammaf_stirling",
    core_math::lgammaf,
    8.0..=35.0
);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_libm,
    bench_metallic_reflect,
    bench_core_math_reflect,
    bench_metallic_rational,
    bench_core_math_rational,
    bench_metallic_stirling,
    bench_core_math_stirling,
);

criterion::criterion_main!(benches);
