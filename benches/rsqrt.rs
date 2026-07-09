mod bench;
// Log-uniform over the positive normals that stay on the main seed branch:
// `PositiveExponents(-1021..=1021)` skips only the rare subnormal (`√x/x`) and
// `x > 2¹⁰²²` (`(4/x)·(0.25·√x)`) ladders, so both sides bench the real kernel.
// No `std`/`libm` legs: neither ships an `rsqrt`.
bench!(
    bench_metallic,
    metallic::rsqrt,
    bench::PositiveExponents(-1021..=1021)
);
bench!(
    bench_core_math,
    core_math::rsqrt,
    bench::PositiveExponents(-1021..=1021)
);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
