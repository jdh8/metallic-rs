mod bench;

bench!(bench_metallic, metallic::tgamma, -10.0..=35.0);
bench!(bench_core_math, core_math::tgamma, -10.0..=35.0);
bench!(bench_libm, libm::tgamma, -10.0..=35.0);

// Per-band splits.  The full-range value-uniform mean is dominated by the cheap
// large-`z` Stirling region, hiding the recurrence band (`[2, 8)`) and reflection
// band (`[-10, -2)`) gaps — see issue #5.  These name-carrying benches surface
// each band's metallic/CORE-MATH ratio separately.
bench!(
    bench_metallic_reflect,
    "metallic::tgamma_reflect",
    metallic::tgamma,
    -10.0..=-2.0
);
bench!(
    bench_core_math_reflect,
    "core_math::tgamma_reflect",
    core_math::tgamma,
    -10.0..=-2.0
);
bench!(
    bench_libm_reflect,
    "libm::tgamma_reflect",
    libm::tgamma,
    -10.0..=-2.0
);
bench!(
    bench_metallic_recur,
    "metallic::tgamma_recur",
    metallic::tgamma,
    2.0..=8.0
);
bench!(
    bench_core_math_recur,
    "core_math::tgamma_recur",
    core_math::tgamma,
    2.0..=8.0
);
bench!(
    bench_libm_recur,
    "libm::tgamma_recur",
    libm::tgamma,
    2.0..=8.0
);
bench!(
    bench_metallic_stirling,
    "metallic::tgamma_stirling",
    metallic::tgamma,
    8.0..=35.0
);
bench!(
    bench_core_math_stirling,
    "core_math::tgamma_stirling",
    core_math::tgamma,
    8.0..=35.0
);
bench!(
    bench_libm_stirling,
    "libm::tgamma_stirling",
    libm::tgamma,
    8.0..=35.0
);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_libm,
    bench_metallic_reflect,
    bench_core_math_reflect,
    bench_libm_reflect,
    bench_metallic_recur,
    bench_core_math_recur,
    bench_libm_recur,
    bench_metallic_stirling,
    bench_core_math_stirling,
    bench_libm_stirling,
);

criterion::criterion_main!(benches);
