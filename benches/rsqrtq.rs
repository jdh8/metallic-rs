#![feature(f128)]

mod bench;
mod bench128;

fn bench_metallic(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "metallic::rsqrtq", || {
        metallic::rsqrtq(bench128::positive_normal())
    });
}

fn bench_core_math(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "core_math::rsqrtq", || {
        core_math::rsqrtq(bench128::positive_normal())
    });
}

criterion::criterion_group!(benches, bench_metallic, bench_core_math);
criterion::criterion_main!(benches);
