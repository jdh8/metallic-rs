mod bench;

bench!(bench_metallic, metallic::lgamma, -10.0..=200.0);
bench!(bench_core_math, core_math::lgamma, -10.0..=200.0);
bench!(bench_libm, libm::lgamma, -10.0..=200.0);

// Per-band splits.  The full-range value-uniform mean is dominated (~79% weight)
// by the cheap `z ≥ 35` region, hiding the recurrence band (`[8, 35)`) and
// reflection band (`[-10, -2)`) gaps — see issue #5.  These name-carrying benches
// surface each band's metallic/CORE-MATH ratio separately.
bench!(
    bench_metallic_reflect,
    "metallic::lgamma_reflect",
    metallic::lgamma,
    -10.0..=-2.0
);
bench!(
    bench_core_math_reflect,
    "core_math::lgamma_reflect",
    core_math::lgamma,
    -10.0..=-2.0
);
bench!(
    bench_libm_reflect,
    "libm::lgamma_reflect",
    libm::lgamma,
    -10.0..=-2.0
);
bench!(
    bench_metallic_recur,
    "metallic::lgamma_recur",
    metallic::lgamma,
    8.0..=35.0
);
bench!(
    bench_core_math_recur,
    "core_math::lgamma_recur",
    core_math::lgamma,
    8.0..=35.0
);
bench!(
    bench_libm_recur,
    "libm::lgamma_recur",
    libm::lgamma,
    8.0..=35.0
);
bench!(
    bench_metallic_stirling,
    "metallic::lgamma_stirling",
    metallic::lgamma,
    35.0..=200.0
);
bench!(
    bench_core_math_stirling,
    "core_math::lgamma_stirling",
    core_math::lgamma,
    35.0..=200.0
);
bench!(
    bench_libm_stirling,
    "libm::lgamma_stirling",
    libm::lgamma,
    35.0..=200.0
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
