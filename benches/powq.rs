#![feature(f128)]

mod bench;
mod bench128;

// CORE-MATH has no `powq`, so the interim baseline is GCC's libquadmath.  It
// is faithful-only (about 1 ulp), so beating it is the floor, not the
// headline; the `core_math::powq` lane and the same-run ratio land with the
// release that binds it.
#[link(name = "quadmath")]
unsafe extern "C" {
    fn powq(x: f128, y: f128) -> f128;
}

// Log-uniform over both arguments, like `atan2q`: `|x| ≤ 2^17` and
// `|y| < 2^9` keep `|y·log2 x|` under 2^13.1, so every result is finite and
// normal, both engines stay on their fast legs, and the sign of `y·log2 x` is
// the coin flip it is in practice.
bench!(
    bench_metallic,
    metallic::powq,
    bench::Exponents(-16..=16),
    bench::Exponents(-16..=8)
);
bench!(
    bench_quadmath,
    "quadmath::powq",
    |x, y| unsafe { powq(x, y) },
    bench::Exponents(-16..=16),
    bench::Exponents(-16..=8)
);

criterion::criterion_group!(benches, bench_metallic, bench_quadmath);
criterion::criterion_main!(benches);
