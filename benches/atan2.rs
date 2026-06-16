mod bench;
// Log-uniform over both args so the ratio `y/x` exponent lands in `[-40, 40]` =
// atan's kernel band.  Under representation-uniform `.., ..` CORE-MATH runs its
// 192-bit accurate path on most pairs (~600 ns vs metallic's ~26 ns), so the
// ratio is not a kernel comparison; the banded draws keep both sides on the fast
// path and drop the inf/NaN/±0 special cases.  See `bench::Exponents`.
bench!(
    bench_metallic,
    metallic::atan2,
    bench::Exponents(-20..=20),
    bench::Exponents(-20..=20)
);
bench!(
    bench_core_math,
    core_math::atan2,
    bench::Exponents(-20..=20),
    bench::Exponents(-20..=20)
);
bench!(
    bench_std,
    f64::atan2,
    bench::Exponents(-20..=20),
    bench::Exponents(-20..=20)
);
bench!(
    bench_libm,
    libm::atan2,
    bench::Exponents(-20..=20),
    bench::Exponents(-20..=20)
);
criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm
);
criterion::criterion_main!(benches);
