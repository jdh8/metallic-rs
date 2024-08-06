use criterion::{BatchSize, Criterion};
use rand::Rng as _;

fn bench(criterion: &mut Criterion, id: &str, f: fn(f32, i32) -> f32) {
    criterion.bench_function(id, |bencher| {
        let rng = &mut rand::thread_rng();

        bencher.iter_batched(
            || (rng.gen::<f32>(), rng.gen_range(-300..300)),
            |(x, n)| f(x, n),
            BatchSize::SmallInput,
        );
    });
}

fn bench_crate(c: &mut Criterion) {
    crate::bench!(bench, c, metallic::f32::ldexp);
}

fn bench_libm(c: &mut Criterion) {
    crate::bench!(bench, c, libm::ldexpf);
}

criterion::criterion_group!(benches, bench_crate, bench_libm);
