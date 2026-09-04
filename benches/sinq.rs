#![feature(f128)]

mod bench;
mod bench128;

// CORE-MATH has no `sinq` yet — its binary128 sine is only a corpus — so the
// interim baseline is GCC's libquadmath.  It is faithful-only (about 1 ulp),
// so beating it is the floor, not the headline; the `core_math::sinq` lane and
// the same-run ratio land with the release that binds it.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn sinq(x: f128) -> f128;
}

// Log-uniform over the direct band and the reduction band, like `atanq`: the
// representation-uniform `..` would spend almost every draw in the huge-argument
// window walk or the tiny-argument fast return.
bench!(bench_metallic, metallic::sinq, bench::Exponents(-20..=20));
bench!(
    bench_quadmath,
    "quadmath::sinq",
    |x| unsafe { sinq(x) },
    bench::Exponents(-20..=20)
);
bench!(bench_std, f128::sin, bench::Exponents(-20..=20));

criterion::criterion_group!(benches, bench_metallic, bench_quadmath, bench_std);
criterion::criterion_main!(benches);
