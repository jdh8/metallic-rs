use criterion::measurement::Measurement;
use criterion::BenchmarkGroup;
use rand::Rng as _;

fn bench<M: Measurement>(group: &mut BenchmarkGroup<M>, name: &str, f: impl Fn(f32) -> f32) {
    group.bench_function(name, |bencher| {
        bencher.iter_batched(
            || rand::thread_rng().gen(),
            &f,
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_cbrt(criterion: &mut criterion::Criterion) {
    let mut group = criterion.benchmark_group("cbrt");
    crate::bench!(bench, &mut group, metallic::f32::cbrt);
    crate::bench!(bench, &mut group, libm::cbrtf);
    crate::bench!(bench, &mut group, f32::cbrt);
}

criterion::criterion_group!(benches, bench_cbrt);
