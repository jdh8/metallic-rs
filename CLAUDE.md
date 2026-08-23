# metallic

This crate provides C math functions written from scratch in Rust, aiming for
correct rounding (error ≤ 0.5 ulp) and performance comparable to or better than
the system math library.  Coverage spans every C99 transcendental in both
precisions plus CORE-MATH's C23 set (`sinpi`…`atan2pi`, `exp2m1`, `exp10m1`,
`log2p1`, `log10p1`, `rsqrt`, `compound`), all correctly rounded (issue #7).
(`compound` is metallic's own: CORE-MATH ships only `compoundf`, so the f64
version is MPFR-verified against a home-grown corpus, like `tgamma`/`lgamma`.)

**Never port polynomial or rational coefficients from CORE-MATH** (or any
other library): CORE-MATH is the *oracle* and a structural reference, never
the source.  Mathematical constants (function-value tables, Taylor terms,
π/ln limbs) are fine when computed independently; fitted minimax coefficients
must be generated with our own tooling (rminimax, Sollya, mpmath) and carry
the generator command in a `///` doc comment.

After updating the codebase, please

- Format the code with `cargo fmt`.
- Run the tests with `cargo test`.
- Propose a clear and descriptive commit message.

Note: do **not** run tests with `--all-features`.  The `_no_fma` feature
disables FMA usage and would cause tests to exercise a different code path than
the default build, producing misleading results.  It also enables the
nightly-only `f128` feature, so it is not a stable-toolchain command.

## Binary128 (`f128`)

Binary128 is opt-in and nightly-only: use `cargo +nightly test --features f128`.
Building the `core-math` oracle for binary128 needs **`CC=clang`** — CORE-MATH's
`hypotq.c` calls `__builtin_addcl`, which GCC does not provide, so a `cc`-built
`core-math-sys` fails to link.  The CI f128 jobs set it; set it locally too.

Public names follow libquadmath and CORE-MATH's `q` suffix (`sqrtq`, `rsqrtq`,
`cbrtq`, `hypotq`). `sqrtq` delegates to Rust's correctly rounded `f128::sqrt`;
the other roots use table-free seeds and exact integer midpoint comparisons for
their final rounding.  `hypotq` needs no Ziv fallback at all: capping the
exponent gap at 56 makes `a² + b²` an exact 384-bit integer, so two exact
comparisons decide the last bit.  The shared 384-bit primitives live in
`src/f128_/uint.rs`.

There is no feasible exhaustive binary128 sweep. The correctness gates are
bit-exact checks against `core_math::*q` on `tests/cases/*q.wc`, deterministic
full-representation samples, and MPFR precision-113 operation + ternary-aware
IEEE subnormalization under `--features "f128 mpfr"`. Keep each corpus count
guard current so a missing or partially parsed file cannot pass vacuously.

Refresh q corpora from `vendor/src/binary128/<fn>/<fn>q.wc` with
`tools/sync-worst-cases.sh`. CORE-MATH remains an oracle and structural
reference: never copy binary128 fitted seed or polynomial tables (`rsqrt9`,
`coef_bind`, `c[][N]`). The current roots deliberately have no fitted tables.

For f128 FMA, call the crate's `fma128` wrapper rather than `f128::mul_add` or a
raw multiply-add. Benchmarks require the `f128` feature and nightly; the
headline is the same-run `metallic::*q / core_math::*q` ratio. The std
`f128::sqrt` lane has the same correct-rounding contract; other std lanes are
faithful-only comparisons.

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
