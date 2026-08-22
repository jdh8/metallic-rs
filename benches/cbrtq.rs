#![feature(f128)]

mod bench;
mod bench128;

fn bench_metallic(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "metallic::cbrtq", || {
        metallic::cbrtq(bench128::normal())
    });
}

fn bench_core_math(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "core_math::cbrtq", || {
        core_math::cbrtq(bench128::normal())
    });
}

fn bench_std(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "f128::cbrt", || bench128::normal().cbrt());
}

criterion::criterion_group!(benches, bench_metallic, bench_core_math, bench_std);
criterion::criterion_main!(benches);
