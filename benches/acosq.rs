#![feature(f128)]

mod bench;
mod bench128;

// Log-uniform magnitudes strictly inside the domain: exponent 0 would be
// `|x| >= 1`, which is the NaN fast-out rather than the pipeline.  The band
// still covers every sector of the `atan2q` reduction the root feeds.
bench!(bench_metallic, metallic::acosq, bench::Exponents(-20..=-1));
bench!(
    bench_core_math,
    core_math::acosq,
    bench::Exponents(-20..=-1)
);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);
criterion::criterion_main!(benches);
