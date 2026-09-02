#![feature(f128)]

mod bench;
mod bench128;

// CORE-MATH has no `tanq` at all — not even a corpus — so the
// interim baseline is GCC's libquadmath.  It is faithful-only (about 1 ulp),
// so beating it is the floor, not the headline; the `core_math::tanq` lane and
// the same-run ratio land with the release that binds it.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn tanq(x: f128) -> f128;
}

// Log-uniform over the direct band and the reduction band, like `atanq`: the
// representation-uniform `..` would spend almost every draw in the huge-argument
// window walk or the tiny-argument fast return.
bench!(bench_metallic, metallic::tanq, bench::Exponents(-20..=20));
bench!(
    bench_quadmath,
    "quadmath::tanq",
    |x| unsafe { tanq(x) },
    bench::Exponents(-20..=20)
);

criterion::criterion_group!(benches, bench_metallic, bench_quadmath);
criterion::criterion_main!(benches);
