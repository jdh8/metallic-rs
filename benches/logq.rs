#![feature(f128)]

mod bench;
mod bench128;

fn bench_metallic(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "metallic::logq", || {
        metallic::logq(bench128::positive_normal())
    });
}

fn bench_core_math(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "core_math::logq", || {
        core_math::logq(bench128::positive_normal())
    });
}

fn bench_std(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "f128::ln", || bench128::positive_normal().ln());
}

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_std);
criterion::criterion_main!(benches);
