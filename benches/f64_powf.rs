mod bench;
// Kernel-active band: the representation-uniform `.., ..` spends almost every
// sample in powf's special-case / gross-overflow fast returns (a negative or
// extreme-magnitude base, or `|y·log₂x|` outside the finite `2^e` range), hiding
// the `log₂ → ×y → exp2` kernel — the same trap `f64_atan` documents.  A positive
// log-uniform base over `[2⁻²⁰, 2²¹)` (so `|log₂x| < 21`) with `y` value-uniform
// in `[-48, 48]` keeps `|y·log₂x| < 1008 < 1024`, so every pair runs the kernel.
// The two ranges are coupled by that product bound; widening one needs the other
// narrowed to stay in range.  See `bench::PositiveExponents`.
bench!(
    bench_metallic,
    metallic::f64::powf,
    bench::PositiveExponents(-20..=20),
    -48.0..=48.0
);
bench!(
    bench_core_math,
    core_math::pow,
    bench::PositiveExponents(-20..=20),
    -48.0..=48.0
);
bench!(
    bench_std,
    f64::powf,
    bench::PositiveExponents(-20..=20),
    -48.0..=48.0
);
bench!(
    bench_libm,
    libm::pow,
    bench::PositiveExponents(-20..=20),
    -48.0..=48.0
);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
