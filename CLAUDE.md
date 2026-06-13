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

## Verification (reproducing CORE-MATH's checks)

Every f64 function reproduces CORE-MATH's per-function check discipline, in
**round-to-nearest only** (metallic's functions are pure RNDN; directed rounding
is out of scope).  Each `tests/<fn>.rs` carries, beyond its dense + full-range
bit-stepping sweep:

- `test_<fn>_worst_cases` — CORE-MATH's `--worst` step: bit-exact vs the
  `core-math` oracle on CORE-MATH's hard-to-round corpus `tests/cases/<fn>.wc`.
  This is the strict correct-rounding gate.  For functions that are not yet
  correctly rounded it is `#[ignore]`d (run with `cargo test -- --ignored`) and
  paired with an active `test_<fn>_worst_faithful` (≤ 1 ulp floor).
- `test_<fn>_vs_mpfr` (`#[cfg(feature = "mpfr")]`) — the independent gold-standard
  cross-check CORE-MATH itself uses, guarding against a shared CORE-MATH bug.
  Run with `cargo test --release --features mpfr`.

Shared helpers live in `tests/common/mod.rs` (`test_worst_univariate`,
`test_worst_bivariate`, `test_worst_faithful`, `mpfr_sweep_univariate`, …).

**Corpora.** `tests/cases/*.wc` are CORE-MATH's worst-case files, committed to
git but excluded from the published crate via `exclude = ["/tests/cases"]` in
`Cargo.toml` (big repo, small dependency).  Refresh them from a `core-math-sys`
checkout with `tools/sync-worst-cases.sh`.

**Correctness status** is tracked in **issue #6** (not #5, which is performance):
the strict gates that are RED, plus the handful of functions that fail even the
faithful floor (genuine bugs at overflow/underflow extremes).  Default
`cargo test` is therefore RED until those are fixed — that is by design.

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
