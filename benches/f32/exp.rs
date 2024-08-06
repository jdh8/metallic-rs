use criterion::{BatchSize, Criterion};
use rand::Rng as _;

fn bench(criterion: &mut Criterion, id: &str, f: fn(f32) -> f32) {
    criterion.bench_function(id, |bencher| {
        bencher.iter_batched(
            || rand::thread_rng().gen_range(-105.0..90.0),
            f,
            BatchSize::SmallInput,
        );
    });
}

fn bench_lib(c: &mut Criterion) {
    crate::bench!(c, metallic::f32::exp);
}

fn bench_sys(c: &mut Criterion) {
    crate::bench!(c, f32::exp);
}

criterion::criterion_group!(benches, bench_lib, bench_sys);
