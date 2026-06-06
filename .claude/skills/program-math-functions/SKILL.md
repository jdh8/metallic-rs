---
name: program-math-functions
description: >-
  Methodology and metallic-rs conventions for implementing math-library
  functions from scratch in Rust — exp, log, sin, pow, erf, and friends. Use
  when adding or improving a function under src/f32/ or src/f64/, doing argument
  reduction, generating minimax/Remez polynomial or rational coefficients
  (rminimax/Sollya, Remez.jl), choosing a polynomial evaluation scheme, applying
  error-free transforms and compensated arithmetic (true FMA available), or
  making a function correctly rounded (≤ 0.5 ulp; Table Maker's Dilemma,
  CORE-MATH, RLIBM). Covers f32 and f64.
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
`src/f32/` and `src/f64/`.

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

Always `cargo bench` against `metallic__f64__<fn>` (the metallic path) and
`core_math__f64__<fn>` (the CORE-MATH path) — not the `f64__<fn>` criterion
group, which benchmarks std.

A real function ties together four ideas, each with a reference file:

- **Exact arithmetic & rounding** → [reference/exact-arithmetic.md](reference/exact-arithmetic.md)
- **Approximation: reduction, transforms, polynomial evaluation** → [reference/approximation.md](reference/approximation.md)
- **Generating coefficients** (rminimax, Remez.jl, Sollya) → [reference/coefficients.md](reference/coefficients.md)
- **Correct rounding** (TMD, CORE-MATH, RLIBM, the proof harness) → [reference/correct-rounding.md](reference/correct-rounding.md)

Read the relevant reference file before writing code in that area — the summaries
below are pointers, not the whole story.

## metallic-rs conventions (read first)

These are facts about *this* repo; follow them so a new function looks like the
existing ones.

**Layout.** Per-type modules: `src/f32/` and `src/f64/`, each with `mod.rs`
(public functions) and `kernel.rs` (inner approximations). The public function
`foo` lives in `mod.rs` as a `pub fn`; its inner approximation lives in
`kernel.rs` as a `pub fn` (e.g. `exp_slope`, `atanh`, `log2`, `exp2`, `rem_pio2`
in `src/f32/kernel.rs` and `src/f64/kernel.rs`). f32 is mature (~28 functions);
f64 is being built out — follow the f32 kernels and the shared `Sum` type as
models.

**Polynomials.** Coefficients are evaluated with `crate::poly(x, &[c0, c1, …])`
(an alias for `fast_polynomial::poly_array`), **low-degree term first** (`c[0]`
is the lowest). `fast_polynomial` already evaluates with a shallow,
Estrin/SIMD-friendly scheme, so you usually do *not* hand-roll Horner and do not
need to choose Horner-vs-Estrin yourself. Rationals use
`fast_polynomial::rational_array`.

**Bit-level helpers.**
- `f64::to_bits()` / `f64::from_bits()` (and the f32 pair) type-pun a bit
  pattern — this is how you read/inject exponent and significand bits (replaces
  C `reinterpret`).
- `kernel::fast_ldexp(x, n)` multiplies by 2ⁿ by adding `n << MANTISSA bits` to
  the bit pattern (fast `scalbn` for in-range results).
- `crate::exp2i(n)` is a `const` 2ⁿ.
- `normalize()` (in each `mod.rs`) breaks a float into `(Sign, Magnitude)` —
  NaN / ∞ / Zero / Normal / Subnormal — so one path handles all magnitudes.

**Double-double (hi + lo).** `src/f64/kernel.rs` defines the `Sum { high, low }`
type with `fast_sum` (Fast2Sum), `Sum::from_sum` (2Sum), `Sum::from_product`,
and `Sum::from_quotient`, plus `Mul`/`Div` operators. Use this where extra
precision is genuinely needed (reduction residual, final add-back) — not
everywhere.

**FMA is available and exact — use it.** Unlike the WASM sibling, Rust's
`f64::mul_add` is a *true*, correctly-rounded fused multiply-add on every target
(hardware when `target_feature=fma`, correct software FMA otherwise). So
FMA-based error-free transforms are the default here:

- `let e = a.mul_add(b, -(a * b));` gives the exact product tail. `Sum::from_product`
  already does this. Prefer it over a Dekker split.
- **Hazard:** the `crate::mul_add` helper (`src/lib.rs`) degrades to `x * y + a`
  when the `fma` target feature is *off*, trading accuracy for speed. It is
  therefore **not** an error-free transform. In EFTs and compensation steps call
  the real `f64::mul_add` (method form) directly; reserve `crate::mul_add` for
  hot polynomial-style spots where a lost low bit doesn't matter. Recommend
  building with `-Ctarget-cpu=native` (see README) so the helper maps to hardware
  FMA.

**Coefficient arrays.** Minimax coefficients go in a slice passed to
`crate::poly`, low-degree first. Keep the generator command in a `///` doc
comment above the array so the coefficients are reproducible (see
[reference/coefficients.md](reference/coefficients.md)).

## Procedure for a new function

1. **Specify the edges first.** Enumerate the special cases: NaN, ±0 (and sign of
   zero in the result), ±∞, overflow/underflow thresholds, domain edges, and
   exact cases. Use `normalize()` to fan these out. Note the function's symmetry —
   even, odd, or through-the-origin — and the algebraic identity you will reduce
   with. Start with `f32`; it is exhaustively verifiable (see step 7).

2. **Reduce the argument** to a small interval where a low-degree polynomial
   converges fast, and **carry the reduction error** in a hi+lo pair (`Sum`, or an
   ad-hoc `(hi, lo)`). `exp2` subtracts the rounded integer `n` and works on
   `x - n`; trig calls `kernel::rem_pio2` (Payne–Hanek over huge arguments)
   returning a quadrant and a reduced angle; `log2` extracts the exponent by an
   integer subtract centred on √2⁄2. See
   [reference/approximation.md](reference/approximation.md).

3. **Pick the approximant form** by symmetry (the heart of the article):
   through-origin ⇒ `f(x) = x·g(x)`, approximate `g`; even ⇒ `f(x) = g(x²)`; odd
   ⇒ `f(x) = x·g(x²)`. Approximating `g` keeps the degree low and makes the
   symmetry *exact*. Decide polynomial vs rational.

4. **Generate the coefficients** over the reduced interval with the right error
   weight (usually relative). Prefer **rminimax/ratapprox**, which optimises
   directly over machine-representable coefficients (`SG` for an f32 kernel, `D`
   for f64); Remez.jl is the quick-exploration / extended-precision alternative.
   Paste the result as a `crate::poly` slice, low-degree first. See
   [reference/coefficients.md](reference/coefficients.md).

5. **Evaluate** with `crate::poly` / `rational_array` and add the **compensation
   terms** that buy the last bits — fold the low word back in with
   `f64::mul_add` or a `Sum`. See
   [reference/exact-arithmetic.md](reference/exact-arithmetic.md) and
   [reference/approximation.md](reference/approximation.md).

6. **Reconstruct** by undoing the reduction, usually via exponent injection:
   `kernel::fast_ldexp(m, n)` or `crate::exp2i`. Handle subnormal outputs and the
   over/underflow ends explicitly (clamp before reduction).

7. **Verify against an oracle.** For `f32`, the test harness sweeps **all 2³² bit
   patterns** and compares to the correctly-rounded `core-math` result — a clean
   sweep *proves* correct rounding (`tests/f32_univariate.rs`). For `f64`, sample
   widely (and use `rug` for ground truth) and track max ulp; rely on published
   hard-to-round tables for the correctly-rounded claim. See
   [reference/correct-rounding.md](reference/correct-rounding.md).

8. **Build, test, commit atomically.** Per `CLAUDE.md`: `cargo fmt`, then
   `cargo test` — **NOT `--all-features`** (the `core-math` feature *replaces*
   metallic functions with CORE-MATH, so it would test the wrong code). Update
   `CHANGELOG.md`. One function (or one coherent improvement) per commit, each
   building and passing tests on its own. `cargo bench` (criterion) to confirm no
   regression vs core-math / std / libm.

## Worked skeleton: `f64::kernel::exp2`

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

And `f64::kernel::log2`, the canonical multiplicative-reduction model:

```rust
pub fn log2(x: f64) -> f64 {
    let i = x.to_bits() as i64;
    // exponent extraction by integer subtract centred on √2/2 ⇒ mantissa near 1
    let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT;
    let x = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);
    // log2(x) = exponent + 2·log2(e)·atanh((x-1)/(x+1)) — odd kernel in a tiny range
    crate::mul_add(2.0 * consts::LOG2_E, atanh((x - 1.0) / (x + 1.0)), exponent as f64)
}
```

Study `exp2`, `log2`, `atanh`, `sin`, `cos`, and `rem_pio2` in `src/f32/kernel.rs`
and `src/f64/kernel.rs` as the canonical models before writing your own.

## External references

- Article this skill is built on — <https://jdh8.org/how-to-program-math-functions/>
- metallic (C/WASM sibling that ports from here) — <https://github.com/jdh8/metallic>
- CORE-MATH (correctly-rounded reference, oracle, hard-to-round tables) — <https://core-math.gitlabpages.inria.fr/>
- rminimax (machine-representable minimax) — <https://gitlab.inria.fr/sfilip/rminimax>; locally cloned at `~/src/rminimax`
- Remez.jl — <https://github.com/simonbyrne/Remez.jl>
- glibc "Errors in Math Functions" (ulp tables) — <https://www.gnu.org/software/libc/manual/html_node/Errors-in-Math-Functions.html>
