#![feature(f128)]

mod bench;
mod bench128;

// Log-uniform over the direct band and the reduction band, like `atanq`: the
// representation-uniform `..` would spend almost every draw in the huge-argument
// window walk or the tiny-argument fast return.
bench!(bench_metallic, metallic::cosq, bench::Exponents(-20..=20));
bench!(bench_core_math, core_math::cosq, bench::Exponents(-20..=20));
bench!(bench_std, f128::cos, bench::Exponents(-20..=20));

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_std);
criterion::criterion_main!(benches);
