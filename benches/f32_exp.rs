use criterion::{criterion_group, criterion_main, Criterion};
use metallic::f32 as lib;

fn bench_exp(exp: impl Fn(f32) -> f32) {
    use core::f32::consts::LN_2;
    let mut x = -160.0 * LN_2;

    while x < 140.0 * LN_2 {
        core::hint::black_box(exp(x));
        x += 1.337;
    }
}

fn bench_lib(criterion: &mut Criterion) {
    criterion.bench_function("bench_lib", |bencher| {
        bencher.iter(|| bench_exp(lib::exp));
    });
}

fn bench_sys(criterion: &mut Criterion) {
    criterion.bench_function("bench_sys", |bencher| {
        bencher.iter(|| bench_exp(f32::exp));
    });
}

criterion_group!(benches, bench_lib, bench_sys);
criterion_main!(benches);
