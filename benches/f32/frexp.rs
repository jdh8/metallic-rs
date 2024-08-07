use criterion::{BatchSize, Criterion};
use rand::Rng as _;

fn bench(criterion: &mut Criterion, name: &str, f: impl Fn(f32) -> (f32, i32)) {
    criterion.bench_function(name, |bencher| {
        bencher.iter_batched(|| rand::thread_rng().gen(), &f, BatchSize::SmallInput);
    });
}

fn bench_crate(c: &mut Criterion) {
    crate::bench!(bench, c, metallic::f32::frexp);
}

fn bench_libm(c: &mut Criterion) {
    crate::bench!(bench, c, libm::frexpf);
}

criterion::criterion_group!(benches, bench_crate, bench_libm);
