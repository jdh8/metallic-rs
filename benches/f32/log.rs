use criterion::measurement::Measurement;
use criterion::BenchmarkGroup;
use rand::Rng as _;

fn bench<M: Measurement>(group: &mut BenchmarkGroup<M>, name: &str, f: impl Fn(f32, f32) -> f32) {
    let rng = &mut rand::thread_rng();

    group.bench_function(name, |bencher| {
        bencher.iter_batched(
            || (rng.gen(), rng.gen()),
            |(x, y)| f(x, y),
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_log(criterion: &mut criterion::Criterion) {
    let mut group = criterion.benchmark_group("f32::log");
    crate::bench!(bench, &mut group, metallic::f32::log);
    crate::bench!(bench, &mut group, f32::log);
}

criterion::criterion_group!(benches, bench_log);
