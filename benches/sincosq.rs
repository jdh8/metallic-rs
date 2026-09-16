#![feature(f128)]

mod bench;
mod bench128;

// Like `sinq` and `cosq`, the interim baseline is GCC's libquadmath — faithful
// only, so beating it is the floor, not the headline.  The std lane has no
// binary128 `sin_cos`; calling `f128::sin` and `f128::cos` in turn would time
// two glibc reductions against metallic's one and say nothing.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn sincosq(x: f128, sin: *mut f128, cos: *mut f128);
}

fn quadmath_sincosq(x: f128) -> (f128, f128) {
    let mut sin = 0.0;
    let mut cos = 0.0;
    unsafe { sincosq(x, &raw mut sin, &raw mut cos) };
    (sin, cos)
}

// Log-uniform over the direct band and the reduction band, like `sinq`.
bench!(
    bench_metallic,
    metallic::sincosq,
    bench::Exponents(-20..=20)
);
bench!(
    bench_quadmath,
    "quadmath::sincosq",
    quadmath_sincosq,
    bench::Exponents(-20..=20)
);

criterion::criterion_group!(benches, bench_metallic, bench_quadmath);
criterion::criterion_main!(benches);
