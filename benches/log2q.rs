#![feature(f128)]

mod bench;
mod bench128;

// CORE-MATH has no `log2q` yet, so the interim baseline is GCC's libquadmath.
// It is faithful-only (about 1 ulp), so beating it is the floor, not the
// headline; the `core_math::log2q` lane and the same-run ratio land with the
// release that binds it.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn log2q(x: f128) -> f128;
}

fn bench_metallic(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "metallic::log2q", || {
        metallic::log2q(bench128::positive_normal())
    });
}

fn bench_quadmath(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "quadmath::log2q", || unsafe {
        log2q(bench128::positive_normal())
    });
}

fn bench_std(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "f128::log2", || {
        bench128::positive_normal().log2()
    });
}

criterion::criterion_group!(benches, bench_metallic, bench_quadmath, bench_std);
criterion::criterion_main!(benches);
