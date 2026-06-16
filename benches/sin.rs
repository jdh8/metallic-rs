mod bench;
// Log-uniform over the kernel band.  Representation-uniform `..` would spend ~49%
// of draws in the `|x| < 2⁻²⁷` fast return and most of the rest in the huge-arg
// Payne–Hanek path (`rem_pio2` switches at `x ≥ 2²⁰`), barely sampling the
// small-angle kernel.  `-26..=19` keeps every draw above the fast return and in
// the medium Cody–Waite reduction.  See `bench::Exponents`.
bench!(bench_metallic, metallic::sin, bench::Exponents(-26..=19));
bench!(bench_core_math, core_math::sin, bench::Exponents(-26..=19));
bench!(bench_std, f64::sin, bench::Exponents(-26..=19));
bench!(bench_libm, libm::sin, bench::Exponents(-26..=19));

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm
);
criterion::criterion_main!(benches);
