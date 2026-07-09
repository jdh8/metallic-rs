mod bench;
// Value-uniform over a few periods: every draw runs the reduction + table
// kernel (no fast-return band to swamp the measurement), and the exact
// integer/half-integer returns are measure-zero.  No `std`/`libm` legs:
// neither ships a `sinpi`.
bench!(bench_metallic, metallic::sinpi, -4.0..=4.0);
bench!(bench_core_math, core_math::sinpi, -4.0..=4.0);

criterion::criterion_group!(benches, bench_metallic, bench_core_math);

criterion::criterion_main!(benches);
