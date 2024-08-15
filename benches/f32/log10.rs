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

fn bench_log10(criterion: &mut criterion::Criterion) {
    let mut group = criterion.benchmark_group("f32::log10");
    crate::bench!(bench, &mut group, metallic::f32::log10);
    crate::bench!(bench, &mut group, libm::log10f);
    crate::bench!(bench, &mut group, f32::log10);
    crate::bench!(bench, &mut group, super::as_safe(core_math_sys::cr_log10f));
}

criterion::criterion_group!(benches, bench_log10);
