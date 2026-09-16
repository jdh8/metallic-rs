#![feature(f128)]

mod bench;

// Neither CORE-MATH nor libquadmath currently provides this entry point.
// The workload matches log1pq's active band. No rounded composition is used
// as a reference, since it would have a different rounding contract.
fn argument() -> f128 {
    let bits = rand::random::<u128>();
    let top = if bits >> 127 != 0 { -1 } else { 13 };
    let e = rand::random_range(-114..=top);
    f128::from_bits(bits & 1 << 127 | ((e + 16383) as u128) << 112 | (bits & (1 << 112) - 1))
}

fn bench_metallic(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "metallic::log10p1q", || {
        metallic::log10p1q(argument())
    });
}

criterion::criterion_group!(benches, bench_metallic);
criterion::criterion_main!(benches);
