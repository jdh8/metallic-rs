#![feature(f128)]
mod bench;

// No CORE-MATH or libquadmath entry point. Both arguments exercise active
// logarithm/exponential paths; separate tiny and general bands show the cost.
fn run(c: &mut criterion::Criterion, name: &str, small: bool) {
    bench::run(c, name, || {
        let a = rand::random::<u128>();
        let b = rand::random::<u128>();
        let e = if small {
            -120 + (a >> 112) as i32 % 101
        } else {
            -18 + (a >> 112) as i32 % 32
        };
        let x = f128::from_bits(
            if e < 0 { a & (1 << 127) } else { 0 }
                | ((e + 16383) as u128) << 112
                | a & ((1 << 112) - 1),
        );
        let ey = if small {
            -e - 16 + ((b >> 112) % 25) as i32
        } else {
            -16 + ((b >> 112) % 25) as i32
        };
        let y =
            f128::from_bits(b & (1 << 127) | ((ey + 16383) as u128) << 112 | b & ((1 << 112) - 1));
        metallic::compoundq(x, y)
    });
}
fn general(c: &mut criterion::Criterion) {
    run(c, "metallic::compoundq", false);
}
fn tiny(c: &mut criterion::Criterion) {
    run(c, "metallic::compoundq_tiny", true);
}
criterion::criterion_group!(benches, general, tiny);
criterion::criterion_main!(benches);
