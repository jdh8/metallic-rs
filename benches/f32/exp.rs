use criterion::{BatchSize, Criterion};
use rand::Rng as _;

fn bench(criterion: &mut Criterion, name: &str, f: impl Fn(f32) -> f32) {
    criterion.bench_function(name, |bencher| {
        bencher.iter_batched(
            || rand::thread_rng().gen_range(-105.0..90.0),
            &f,
            BatchSize::SmallInput,
        );
    });
}

fn bench_crate(c: &mut Criterion) {
    crate::bench!(bench, c, metallic::f32::exp);
}

fn bench_libm(c: &mut Criterion) {
    crate::bench!(bench, c, libm::expf);
}

fn bench_std(c: &mut Criterion) {
    crate::bench!(bench, c, f32::exp);
}

criterion::criterion_group!(benches, bench_crate, bench_libm, bench_std);
