fn main() {
    // Representation-uniform bit sweep over all f64 (finite).
    let mut bad = [0u64; 3];
    let mut worst = [0i64; 3];
    let mut n = 0u64;
    let mut i: u64 = 0;
    while i <= u64::MAX - ((1u64 << 33) - 1337) {
        let x = f64::from_bits(i);
        if x.is_finite() {
            n += 1;
            for (k, (f, g)) in [
                (
                    metallic::sin as fn(f64) -> f64,
                    core_math::sin as fn(f64) -> f64,
                ),
                (metallic::cos, core_math::cos),
                (metallic::tan, core_math::tan),
            ]
            .into_iter()
            .enumerate()
            {
                let a = f(x);
                let b = g(x);
                if !(a.is_nan() && b.is_nan()) {
                    let d = (a.to_bits() as i64 - b.to_bits() as i64).abs();
                    if d != 0 {
                        bad[k] += 1;
                        if d > worst[k] {
                            worst[k] = d;
                            if bad[k] <= 3 {
                                eprintln!("  k{k} {x:e}: {a:e} vs {b:e} ({d})");
                            }
                        }
                    }
                }
            }
        }
        i += (1u64 << 33) - 1337;
    }
    eprintln!(
        "bit-sweep n={n}: sin bad={} (worst {}), cos bad={} (worst {}), tan bad={} (worst {})",
        bad[0], worst[0], bad[1], worst[1], bad[2], worst[2]
    );
}
