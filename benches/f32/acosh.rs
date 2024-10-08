use criterion::measurement::Measurement;
use criterion::BenchmarkGroup;
use rand::Rng as _;

fn bench<M: Measurement>(group: &mut BenchmarkGroup<M>, name: &str, f: impl Fn(f32) -> f32) {
    group.bench_function(name, |bencher| {
        bencher.iter_batched(
            || rand::thread_rng().gen::<f32>().abs(),
            &f,
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_acosh(criterion: &mut criterion::Criterion) {
    let mut group = criterion.benchmark_group("f32::acosh");
    crate::bench!(bench, &mut group, metallic::f32::acosh);
    crate::bench!(bench, &mut group, libm::acoshf);
    crate::bench!(bench, &mut group, f32::acosh);
    crate::bench!(bench, &mut group, core_math::acoshf);
}

criterion::criterion_group!(benches, bench_acosh);
