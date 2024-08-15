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

fn bench_hypot(criterion: &mut criterion::Criterion) {
    let mut group = criterion.benchmark_group("f32::hypot");
    crate::bench!(bench, &mut group, metallic::f32::hypot);
    crate::bench!(bench, &mut group, libm::hypotf);
    crate::bench!(bench, &mut group, f32::hypot);
    crate::bench!(bench, &mut group, |x, y| unsafe { core_math_sys::hypotf(x, y) });
}

criterion::criterion_group!(benches, bench_hypot);
