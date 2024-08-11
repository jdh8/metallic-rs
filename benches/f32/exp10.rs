use criterion::measurement::Measurement;
use criterion::BenchmarkGroup;
use rand::Rng as _;

fn bench<M: Measurement>(group: &mut BenchmarkGroup<M>, name: &str, f: impl Fn(f32) -> f32) {
    group.bench_function(name, |bencher| {
        bencher.iter_batched(
            || rand::thread_rng().gen_range(-50.0..40.0),
            &f,
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_exp10(criterion: &mut criterion::Criterion) {
    let mut group = criterion.benchmark_group("f32::exp10");
    crate::bench!(bench, &mut group, metallic::f32::exp10);
    crate::bench!(bench, &mut group, libm::exp10f);
}

criterion::criterion_group!(benches, bench_exp10);
