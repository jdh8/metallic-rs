use criterion::{BatchSize, Criterion};
use metallic::f32 as lib;
use rand::Rng;

fn bench_exp(criterion: &mut Criterion, id: &str, f: fn(f32) -> f32) {
    criterion.bench_function(id, |bencher| {
        bencher.iter_batched(
            || rand::thread_rng().gen_range(-105.0..90.0),
            f,
            BatchSize::SmallInput,
        );
    });
}

fn bench_lib(c: &mut Criterion) {
    bench_exp(c, "lib_exp", lib::exp);
}

fn bench_sys(c: &mut Criterion) {
    bench_exp(c, "sys_exp", f32::exp);
}

criterion::criterion_group!(benches, bench_lib, bench_sys);
