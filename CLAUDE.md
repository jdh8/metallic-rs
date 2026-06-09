# metallic

This crate provides C math functions written from scratch in Rust, aiming for
correct rounding (error ≤ 0.5 ulp) and performance comparable to or better than
the system math library.

After updating the codebase, please

- Format the code with `cargo fmt`.
- Run the tests with `cargo test`.
- Propose a clear and descriptive commit message.

Note: do **not** run tests with `--all-features`.  The `_no_fma` feature
disables FMA usage and would cause tests to exercise a different code path than
the default build, producing misleading results.

## Fused multiply-add

Never call `f32::mul_add` / `f64::mul_add` directly (clippy-denied), and never
hand-write a raw `a * b + c` where a fused multiply-add is intended
(`suboptimal_flops` is denied):

- Need an **exact** FMA — error-free transforms, residual extraction,
  double-double compensation? Use `crate::fma` (f64) or `crate::fmaf` (f32).
- Otherwise — hot polynomial spots where a lost low bit is fine? Use
  `crate::fast_mul_add` instead of `a * b + c`.

There is no f32 `fast_mul_add` yet: f32 hot paths promote to f64 and call
`crate::fast_mul_add`.  If a genuine f32-precision fast multiply-add is ever
needed, add a `fast_mul_addf` (mirroring `fmaf`) rather than writing raw
`a * b + c` in f32.
