---
name: program-math-functions
description: >-
  Methodology and metallic-rs conventions for implementing math-library
  functions from scratch in Rust — exp, log, sin, pow, erf, and friends. Use
  when adding or improving a function under src/f32_/ or src/f64_/, doing
  argument reduction, generating minimax/Remez polynomial or rational
  coefficients (rminimax/Sollya, Remez.jl), choosing a polynomial evaluation
  scheme, applying error-free transforms and compensated arithmetic (true FMA
  available), making a function correctly rounded (≤ 0.5 ulp; Table Maker's
  Dilemma, CORE-MATH, RLIBM), or benchmarking and optimizing a function's
  performance against CORE-MATH (criterion benches, Ziv fallback rates,
  branchless rewrites, issue #5). Covers f32 and f64.
---

# Programming math functions

metallic-rs is the original Rust math library; its **target is correct rounding
(≤ 0.5 ulp)** — the nearest representable, every time — and a merely-faithful
function (< 1 ulp) is treated as a bug to fix, not shipped as "done".
Implementations are *original*: not ported from musl, fdlibm, or glibc. The
sibling C project [metallic](https://github.com/jdh8/metallic) ports *from here*,
adapting for WASM (no scalar FMA, round-to-nearest only); this skill is the Rust
source of that lineage, distilled from
<https://jdh8.org/how-to-program-math-functions/> and the conventions already in
`src/f32_/` and `src/f64_/`.

**Where the project stands:** every f32 *and* f64 function is already correctly
rounded, with active bit-exact worst-case gates (issue #6, closed 2026-06-14) —
a red gate is a regression. The open front is **performance** (issue #5):
closing the remaining gaps to CORE-MATH without breaking a gate. Start
performance work at [reference/performance.md](reference/performance.md).

**Performance target: beat CORE-MATH.** CORE-MATH is the correctly-rounded
reference implementation. Matching its throughput is the floor; the real goal is
to run *faster*. The main lever: CORE-MATH supports all four IEEE rounding modes
and must detect the current mode at runtime, while metallic-rs targets
**round-to-nearest only**. This unlocks:

- **Tighter polynomial bounds.** Under RTN the error budget is exactly ½ ulp;
  other modes need more headroom. A coefficients search can be tighter, sometimes
  saving a degree.
- **No rounding-mode branching.** CORE-MATH often checks `fegetround()` in its
  final rounding step. We skip that branch entirely.
- **Faster Ziv refinement.** A first-pass approximation only needs to clear the
  RTN tie-breaking threshold, not the wider fence that covers directed rounding.

Always bench with `RUSTFLAGS=-Ctarget-cpu=x86-64-v3 cargo bench --bench <fn>`
(the host default has no FMA) and compare `metallic::<fn>` against
`core_math::<fn>` — not `f64::<fn>`, which benchmarks std. f32 benches carry
the `f` suffix (`metallic::expf`). `python3 tools/bench_ratio.py median` turns
the criterion output into the paired ratio table.

A real function ties together five ideas, each with a reference file:

- **Exact arithmetic & rounding** → [reference/exact-arithmetic.md](reference/exact-arithmetic.md)
- **Approximation: reduction, transforms, polynomial evaluation** → [reference/approximation.md](reference/approximation.md)
- **Generating coefficients** (rminimax, Remez.jl, Sollya, `tools/gen_*.py`) → [reference/coefficients.md](reference/coefficients.md)
- **Correct rounding** (TMD, the gate harness, Ziv soundness, the hard tier) → [reference/correct-rounding.md](reference/correct-rounding.md)
- **Performance** (the optimization loop, diagnosis tree, dead-ends) → [reference/performance.md](reference/performance.md)

Read the relevant reference file before writing code in that area — the summaries
below are pointers, not the whole story.

## metallic-rs conventions (read first)

These are facts about *this* repo; follow them so a new function looks like the
existing ones.

**Layout.** Per-type modules `src/f32_/` and `src/f64_/` (trailing underscore —
the bare names `f32`/`f64` would clash with the primitive types), each a
`name.rs` + `name/` pair, **no `mod.rs`**. Functions are grouped by family into
files — `exp.rs`, `log.rs`, `trig.rs`, `hyp.rs`, `gamma.rs`, `misc.rs`, … — and
`src/f64_/double.rs` holds the double-double type and bit helpers. A family file
holds *both* the public function and its inner approximations; the public API is
re-exported flat, libm-style, at the crate root: `crate::exp2`, `crate::log2`
(f64 keeps the bare C name), `crate::exp2f` (f32 gets the `f` suffix). Both
precision trees are built out and correctly rounded — study the existing family
files and the shared `DoubleDouble` type as models.

**Polynomials.** Evaluate with `crate::poly(x, &[c0, c1, …])` (alias for
`fast_polynomial::poly_array`), **low-degree term first**; rationals use
`fast_polynomial::rational_array`. You do *not* hand-roll Horner or choose
Horner-vs-Estrin — see [reference/approximation.md](reference/approximation.md)
§ Evaluation schemes for why.

**Bit-level helpers.**
- `f64::to_bits()` / `f64::from_bits()` (and the f32 pair) type-pun a bit
  pattern — this is how you read/inject exponent and significand bits (replaces
  C `reinterpret`).
- `fast_ldexp(x, n)` (in `src/f64_/double.rs`) multiplies by 2ⁿ by adding
  `n << MANTISSA bits` to the bit pattern (fast `scalbn` for in-range results).
- `crate::exp2i(n)` is a `const` 2ⁿ.
- `normalize()` (in `src/f32_/misc.rs` and `src/f64_/misc.rs`) breaks a float
  into `(Sign, Magnitude)` — NaN / ∞ / Zero / Normal / Subnormal — so one path
  handles all magnitudes.

**Double-double (hi + lo).** `src/f64_/double.rs` defines the
`DoubleDouble { high, low }` type with `fast_sum` (Fast2Sum), `from_sum` (2Sum),
`from_product`, `from_quotient`, `recip`, plus `Add`/`Mul`/`Div` operators. Carry
it only where extra precision genuinely pays — see the laziness ladder below
(rung 3).

**FMA is true and exact — use it.** Rust's `f64::mul_add` is a correctly-rounded
FMA on every target; the crate wraps it as `crate::fma` / `crate::fmaf`.
**Rule:** error-free transforms and compensation → `crate::fma` / `crate::fmaf`;
hot polynomial spots where a lost low bit is fine → `crate::fast_mul_add`; never
the clippy-denied builtin and never a raw `a * b + c`. The `fast_mul_add` helper
degrades to `x * y + c` without the `fma` target feature, so it is **not** an
error-free transform. Full treatment and the EFT recipes:
[reference/exact-arithmetic.md](reference/exact-arithmetic.md). (CLAUDE.md
restates this rule in always-loaded context.)

**Coefficient arrays.** Minimax coefficients go in a slice passed to
`crate::poly`, low-degree first. Keep the generator command in a `///` doc
comment above the array so the coefficients are reproducible (see
[reference/coefficients.md](reference/coefficients.md)).

## Do the least that clears ½ ulp

Correct rounding is the only non-negotiable; everything else is the *least* code
that provably reaches it. Stop at the first rung that holds — the rest of this
skill cross-references these rungs instead of repeating "only where needed":

1. **Reuse before writing** — an existing kernel or helper (`exp_slope`,
   `atanh`, `rem_pio2`, `ln_fast`, the `DoubleDouble` type) before a new one.
2. **`crate::poly` before hand-rolling** Horner/Estrin — `fast_polynomial`
   already gives a shallow, SIMD-friendly chain.
3. **Plain `f64` before a `DoubleDouble`** — carry a hi+lo pair only where
   cancellation bites: the reduction residual and the final add-back.
4. **Polynomial before rational** — a division is a real cost; reach for
   `rational_array` only when a polynomial needs an uncomfortable degree.
5. **Published worst cases before generating your own** — use CORE-MATH /
   Lefèvre–Muller tables; generate (slow — see
   [reference/correct-rounding.md](reference/correct-rounding.md)) only when no
   table covers the function.
6. **Add precision only when the budget forces it** — fold a low word, add a
   Ziv level, or raise a degree *after* a ulp failure or an error bound demands
   it, never pre-emptively.

## Procedure for a new function

1. **Specify the edges first.** Enumerate the special cases: NaN, ±0 (and sign of
   zero in the result), ±∞, overflow/underflow thresholds, domain edges, and
   exact cases. Use `normalize()` to fan these out. Note the function's symmetry —
   even, odd, or through-the-origin — and the algebraic identity you will reduce
   with. Start with `f32`; it is exhaustively verifiable (see step 7).

2. **Reduce the argument** to a small interval where a low-degree polynomial
   converges fast, and **carry the reduction error** in a hi+lo pair
   (`DoubleDouble`, or an ad-hoc `(hi, lo)`) — but only where cancellation bites
   (ladder rung 3). `exp2` subtracts the rounded integer `n` and works on
   `x - n`; trig calls `rem_pio2` (`src/{f32_,f64_}/trig.rs`, Payne–Hanek over
   huge arguments) returning a quadrant and a reduced angle; `log2` extracts the
   exponent by an integer subtract centred on √2⁄2. See
   [reference/approximation.md](reference/approximation.md).

3. **Pick the approximant form** by symmetry (the heart of the article):
   through-origin ⇒ `f(x) = x·g(x)`, approximate `g`; even ⇒ `f(x) = g(x²)`; odd
   ⇒ `f(x) = x·g(x²)`. Approximating `g` keeps the degree low and makes the
   symmetry *exact*. Decide polynomial vs rational (rung 4).

4. **Generate the coefficients** over the reduced interval with the right error
   weight (usually relative). Prefer **rminimax/ratapprox**, which optimises
   directly over machine-representable coefficients (`SG` for an f32 kernel, `D`
   for f64); Remez.jl is the quick-exploration / extended-precision alternative.
   Paste the result as a `crate::poly` slice, low-degree first. See
   [reference/coefficients.md](reference/coefficients.md).

5. **Evaluate** with `crate::poly` / `rational_array` and add the **compensation
   terms** that buy the last bits — fold the low word back in with `crate::fma`
   or a `DoubleDouble`, but only when the error budget forces it (rung 6). See
   [reference/exact-arithmetic.md](reference/exact-arithmetic.md) and
   [reference/approximation.md](reference/approximation.md).

6. **Reconstruct** by undoing the reduction, usually via exponent injection:
   `fast_ldexp(m, n)` or `crate::exp2i`. Handle subnormal outputs and the
   over/underflow ends explicitly (clamp before reduction).

7. **Verify against an oracle.** For `f32`, each function's test sweeps **all
   2³² bit patterns** against the `core-math` oracle (`common::test_all_f32`) —
   a clean sweep *proves* correct rounding. For `f64`, the per-function
   `tests/<fn>.rs` gates (CORE-MATH worst-case corpus, MPFR sweep) are the
   proof; if you added or changed a Ziv fast leg or gate, its in-source
   `ziv_soundness` certification ships **in the same commit**. See
   [reference/correct-rounding.md](reference/correct-rounding.md).

8. **Build, test, commit atomically.** Per `CLAUDE.md`: `cargo fmt`, then
   `cargo test` — **NOT `--all-features`** (the `_no_fma` feature disables FMA
   and would exercise a different code path, producing misleading results).
   One function (or one coherent improvement) per commit, each building and
   passing tests on its own. Bench per
   [reference/performance.md](reference/performance.md) to confirm no
   regression vs CORE-MATH, and quote the same-run ratio in the commit message.

## Worked skeleton: a reduction → reconstruct `exp2`

Simplified to show the procedure's bones (the shipped `src/f64_/exp.rs::exp2`
adds Ziv refinement on top):

```rust
pub fn exp2(x: f64) -> f64 {
    if x < (f64::MIN_EXP - 1).into() { return 0.0; }       // 1. underflow edge
    if x > f64::MAX_EXP.into()       { return f64::INFINITY; } // 1. overflow edge

    let n = x.round_ties_even();                            // 2. reduce: r = x - n ∈ [-½, ½]
    let x = crate::poly(x - n, &[                           // 4+5. minimax 2^r on [-½, ½]
        1.0, 6.931_471_880_289_533e-1, 2.402_265_108_421_173_5e-1,
        5.550_357_105_498_874_4e-2, 9.618_030_771_171_498e-3,
        1.339_086_685_300_951e-3, 1.546_973_499_989_028_8e-4,
    ]);
    // SAFETY: n ∈ (f64::MIN_EXP - 1) ..= f64::MAX_EXP
    fast_ldexp(x, unsafe { n.to_int_unchecked() })          // 6. reconstruct: ×2^n
}
```

And `log2` (`src/f64_/log.rs`), the canonical multiplicative-reduction model:

```rust
pub fn log2(x: f64) -> f64 {
    let i = x.to_bits() as i64;
    // exponent extraction by integer subtract centred on √2/2 ⇒ mantissa near 1
    let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;
    let x = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);
    // log2(x) = exponent + 2·log2(e)·atanh((x-1)/(x+1)) — odd kernel in a tiny range
    crate::fast_mul_add(2.0 * consts::LOG2_E, atanh((x - 1.0) / (x + 1.0)), exponent as f64)
}
```

Study `exp2`, `log2`, `atanh`, `sin`, `cos`, and `rem_pio2` in `src/f32_/` and
`src/f64_/` (e.g. `exp.rs`, `log.rs`, `trig.rs`, `hyp.rs`) as the canonical models
before writing your own.

## External references

- Article this skill is built on — <https://jdh8.org/how-to-program-math-functions/>
- metallic (C/WASM sibling that ports from here) — <https://github.com/jdh8/metallic>
- CORE-MATH (correctly-rounded reference, oracle, hard-to-round tables) — <https://core-math.gitlabpages.inria.fr/>
- rminimax (machine-representable minimax) — <https://gitlab.inria.fr/sfilip/rminimax>; locally cloned at `~/src/rminimax`
- Remez.jl — <https://github.com/simonbyrne/Remez.jl>
- glibc "Errors in Math Functions" (ulp tables) — <https://www.gnu.org/software/libc/manual/html_node/Errors-in-Math-Functions.html>
