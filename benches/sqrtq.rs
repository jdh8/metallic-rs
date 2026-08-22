#![feature(f128)]

mod bench;
mod bench128;

fn bench_metallic(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "metallic::sqrtq", || {
        metallic::sqrtq(bench128::positive_normal())
    });
}

fn bench_core_math(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "core_math::sqrtq", || {
        core_math::sqrtq(bench128::positive_normal())
    });
}

fn bench_std(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "f128::sqrt", || {
        bench128::positive_normal().sqrt()
    });
}

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_std);
criterion::criterion_main!(benches);
