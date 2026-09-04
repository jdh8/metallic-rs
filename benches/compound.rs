mod bench;

// Match compoundf's base and exponent bands. No comparator exposes the same
// operation: rounding 1 + x before a pow call would change its contract.
bench!(bench_metallic, metallic::compound, -0.5..=2.0, -50.0..=50.0);

criterion::criterion_group!(benches, bench_metallic);
criterion::criterion_main!(benches);
