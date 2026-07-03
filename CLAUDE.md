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
  This is the strict correct-rounding gate.
- `test_<fn>_vs_mpfr` (`#[cfg(feature = "mpfr")]`) — the independent gold-standard
  cross-check CORE-MATH itself uses, guarding against a shared CORE-MATH bug.
  Run with `cargo test --release --features mpfr`.

Shared helpers live in `tests/common/mod.rs` (`test_worst_univariate`,
`test_worst_bivariate`, `test_worst_faithful`, `mpfr_sweep_univariate`, …).

**Corpora.** `tests/cases/*.wc` are CORE-MATH's worst-case files, committed to
git but excluded from the published crate via `exclude = ["/tests/cases"]` in
`Cargo.toml` (big repo, small dependency).  Refresh them from a `core-math-sys`
checkout with `tools/sync-worst-cases.sh`.

**Correctness status: complete.** Every f32 and f64 function is correctly
rounded and every strict gate is active (issue #6, closed 2026-06-14).  Default
`cargo test` is GREEN; a red gate is a **regression** to fix before anything
else, never something to `#[ignore]`.

**Ziv-gate soundness rule.** A new or changed fast leg or Ziv gate ships its
in-source `mod ziv_soundness` MPFR certification (worst `|err|/gate < 0.5`,
i.e. ≥ 2× margin) **in the same commit**, with the printed margin quoted in the
commit message.  See the skill's `reference/correct-rounding.md` for the
pattern (`src/f64_/log.rs` has the model modules).

## Performance

Performance work is tracked in **issue #5** (#6 was correctness).  Bench with
`RUSTFLAGS=-Ctarget-cpu=x86-64-v3 cargo bench --bench <fn>` (the CI-canonical
flag — the host default has **no FMA**), then run `python3
tools/bench_ratio.py median` for the paired table.  The headline number is the
**same-run CORE-MATH ratio** (`metallic::<fn>` / `core_math::<fn>`): CORE-MATH
shares metallic's ≤ 0.5 ulp contract, while std/libm are faithful-only and do
less work.  Never compare across runs (CORE-MATH is a moving baseline) and
check box load *and* memory pressure before trusting absolute numbers.  The
full playbook — diagnosis tree, proven patterns, falsified dead-ends — is the
skill's `reference/performance.md`.

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
