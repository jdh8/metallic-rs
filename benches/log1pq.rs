#![feature(f128)]

mod bench;
mod bench128;

// CORE-MATH has no `log1pq` yet, so the interim baseline is GCC's libquadmath.
// It is faithful-only (about 1 ulp), so beating it is the floor, not the
// headline; the `core_math::log1pq` lane and the same-run ratio land with the
// release that binds it.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn log1pq(x: f128) -> f128;
}

/// A random sign and significand at an exponent uniform over `[2^-114, 2^14)`
/// — below 2^-113 the result is the argument — kept above −1 when negative.
fn argument() -> f128 {
    let bits = rand::random::<u128>();
    let top = if bits >> 127 != 0 { -1 } else { 13 };
    let e = rand::random_range(-114..=top);

    f128::from_bits(bits & 1 << 127 | ((e + 16383) as u128) << 112 | (bits & (1 << 112) - 1))
}

fn bench_metallic(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "metallic::log1pq", || {
        metallic::log1pq(argument())
    });
}

fn bench_quadmath(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "quadmath::log1pq", || unsafe {
        log1pq(argument())
    });
}

fn bench_std(criterion: &mut criterion::Criterion) {
    bench::run(criterion, "f128::ln_1p", || argument().ln_1p());
}

criterion::criterion_group!(benches, bench_metallic, bench_quadmath, bench_std);
criterion::criterion_main!(benches);
