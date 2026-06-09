mod bench;
// Log-uniform over the kernel's active band: representation-uniform `..` would
// spend ~96% of samples in `atan_dd`'s `|x| < 2⁻⁴⁰` / `|x| > 2⁴⁰` fast returns,
// hiding the kernel cost.  See `bench::Exponents`.
bench!(bench_metallic, metallic::atan, bench::Exponents(-40..=39));
bench!(bench_core_math, core_math::atan, bench::Exponents(-40..=39));
bench!(bench_std, f64::atan, bench::Exponents(-40..=39));
bench!(bench_libm, libm::atan, bench::Exponents(-40..=39));
criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm
);
criterion::criterion_main!(benches);
