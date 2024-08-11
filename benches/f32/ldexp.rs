use criterion::measurement::Measurement;
use criterion::BenchmarkGroup;
use rand::Rng as _;

fn bench<M: Measurement>(group: &mut BenchmarkGroup<M>, name: &str, f: impl Fn(f32, i32) -> f32) {
    let rng = &mut rand::thread_rng();

    group.bench_function(name, |bencher| {
        bencher.iter_batched(
            || (rng.gen(), rng.gen_range(-300..300)),
            |(x, n)| f(x, n),
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_ldexp(criterion: &mut criterion::Criterion) {
    let mut group = criterion.benchmark_group("f32::ldexp");
    crate::bench!(bench, &mut group, metallic::f32::ldexp);
    crate::bench!(bench, &mut group, libm::ldexpf);
}

criterion::criterion_group!(benches, bench_ldexp);
