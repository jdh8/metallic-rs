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

fn bench_ln_1p(criterion: &mut criterion::Criterion) {
    let mut group = criterion.benchmark_group("f32::ln_1p");
    crate::bench!(bench, &mut group, metallic::f32::ln_1p);
    crate::bench!(bench, &mut group, libm::log1pf);
    crate::bench!(bench, &mut group, f32::ln_1p);
    crate::bench!(bench, &mut group, super::as_safe(core_math_sys::cr_log1pf));
}

criterion::criterion_group!(benches, bench_ln_1p);
