use criterion::{criterion_group, Criterion};
use metallic::f32 as lib;

fn bench_exp(f: impl Fn(f32) -> f32) {
    let mut x = 100.0;

    #[allow(clippy::while_float)]
    while core::hint::black_box(f(x)) > 0.0 {
        x -= 1.337e-2;
    }
}

fn bench_lib(c: &mut Criterion) {
    c.bench_function("bench_lib", |b| {
        b.iter(|| bench_exp(lib::exp));
    });
}

fn bench_sys(c: &mut Criterion) {
    c.bench_function("bench_sys", |b| {
        b.iter(|| bench_exp(f32::exp));
    });
}

criterion_group!(benches, bench_lib, bench_sys);
