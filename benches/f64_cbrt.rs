macro_rules! bench {
    ($name:ident, $callback:expr) => {
        fn $name(criterion: &mut criterion::Criterion) {
            criterion.bench_function(stringify!($callback), |bencher| {
                bencher.iter(|| $callback(rand::random()));
            });
        }
    };
}

bench!(bench_metallic, metallic::f64::cbrt);
bench!(bench_core_math, core_math::cbrt);
bench!(bench_std, f64::cbrt);
bench!(bench_libm, libm::cbrt);

criterion::criterion_group!(
    benches,
    bench_metallic,
    bench_core_math,
    bench_std,
    bench_libm,
);

criterion::criterion_main!(benches);
