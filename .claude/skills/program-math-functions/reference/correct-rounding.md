# Correct rounding

**The target for this library is correct rounding: error ≤ 0.5 ulp** — the returned
value is *the* nearest representable to the exact result (ties to even),
indistinguishable from computing in infinite precision and rounding once. This is a
step up from *faithful* rounding (< 1 ulp, i.e. one of the two nearest). metallic-rs
takes ≤ 0.5 ulp as its primary goal and treats a merely-faithful function as a bug
to fix; the C sibling [metallic](https://github.com/jdh8/metallic) is moving the
same way and ports its kernels from here. A function not yet correctly rounded
should be labelled WIP, not shipped as "done".

**Baseline: this goal is met.** Every f32 and f64 function is correctly rounded
(issue #6, closed 2026-06-14) and every strict worst-case gate is active — a red
gate in `cargo test` is a **regression**, not a work-in-progress marker. New work
must keep the gates green; the open front is performance (issue #5, see
[performance.md](performance.md)).

## The Table Maker's Dilemma

To return the correctly-rounded `f(x)` you must decide which side of a rounding
boundary the *exact* `f(x)` lies on. Under round-to-nearest the boundary is the
midpoint between two representables. The exact value can sit arbitrarily close to
such a midpoint, so **no fixed working precision is guaranteed in advance** to
resolve every input — that is the dilemma. The "hardness" of an input is the length
of the run of identical bits just past the rounding position (a long run of 0s or
1s means the value hugs a boundary). The maximum hardness over all inputs is the
worst case, and it tells you how much precision is enough.

Two ways to defeat it:

1. **Ziv's onion-peeling** (IBM Accurate Math, glibc, CORE-MATH). Evaluate with a
   fast approximation that *also* produces a rigorous error bound ε. If
   `[y − ε, y + ε]` does not straddle a rounding boundary, return `y`; otherwise
   fall back to a slower, higher-precision level (in metallic-rs that second level
   is a `DoubleDouble` evaluation — see [exact-arithmetic.md](exact-arithmetic.md)).
   A couple of levels handle all but a handful of inputs. The catch is that each
   level needs a *correct* ε — prove it with Gappa or interval arithmetic, don't
   guess.

2. **Direct LP construction** (RLIBM). Don't approximate `f` and hope it rounds
   right — encode "the rounded output equals the correctly-rounded value for every
   input" as linear constraints and **solve for the polynomial** (counterexample-
   guided). Because correctness is only required *after* the final rounding, there
   is slack a minimax fit lacks, so the polynomial is often **lower degree (faster)**
   than a faithful one. Papers: RLIBM (POPL'21)
   <https://people.cs.rutgers.edu/~sn349/papers/rlibm-popl-2021.pdf>, RLIBM-ALL
   <https://arxiv.org/abs/2108.06756>, RLibm-MultiRound (PLDI'25)
   <https://people.cs.rutgers.edu/~santosh.nagarakatte/papers/rlibm-multiround-pldi-2025.pdf>.

## The single-mode advantage

CORE-MATH and RLIBM-ALL pay to be correct in **all four** IEEE rounding modes and
many formats; Rust (like WASM) effectively has exactly **one** —
round-to-nearest-ties-to-even, with no portable `fesetround`. SKILL.md spells out
the three *speed* levers this unlocks; the point for *correctness* is narrower:
**the proof obligation is one mode, not four.** Only RN midpoints are hard cases
(not the directed-mode boundaries as well), so there is more approximation slack,
and none of the round-to-odd-in-two-extra-bits machinery multi-mode libraries carry
is required — round-to-odd stays a tool for collapsing a `DoubleDouble`, not a
requirement. FMA is free here too (`crate::fma`/`crate::fmaf`); see
[exact-arithmetic.md](exact-arithmetic.md) for the rule.

## Sources to reference and oracles to test against

- **CORE-MATH** <https://core-math.gitlabpages.inria.fr/> (MIT) — correctly-rounded
  reference for 50+ functions, upstreamed into glibc 2.42+. Exposed in Rust through
  the **`core-math` crate**, which this repo uses as its **test oracle** (its answer
  *is* correctly rounded), as a **design reference**, and for its **hard-to-round /
  worst-case databases**.
- **`rug`** (Rust MPFR bindings) — arbitrary-precision, correctly-rounded ground
  truth for f64 where `core-math` lacks the function: compute `f(x)` to ~200 bits,
  round to the target, compare.

## How you *prove* a function is correctly rounded

**`f32` (binary32): exhaustive — and the harness already exists.** Only 2³² bit
patterns exist. Each f32 function has a per-function test file (`tests/all/f32_/sinf.rs`,
`tests/all/f32_/expf.rs`, …) whose core is one line: `common::test_all_f32(metallic::sinf,
core_math::sinf)` — it loops **every** `f32` and compares against the `core-math`
oracle via the `Identity` trait in `tests/all/common.rs` (bit-equal, NaNs equal).
A clean sweep is a *proof* of correct rounding and is the CI gate; it runs
natively in minutes. Bivariate functions use `common::test_bivariate_cases`;
`common::truncate_errors` caps the report at 250 mismatches so a regression fails
fast.

**`f64` (binary64): cannot brute-force** (2⁶⁴ inputs) — so the repo reproduces
CORE-MATH's own per-function check discipline. Each `tests/all/f64_/<fn>.rs` carries:

- **`test_<fn>_worst_cases`** — the strict correct-rounding gate: bit-exact vs
  the `core-math` oracle on CORE-MATH's hard-to-round corpus
  `tests/cases/<fn>.wc` (their BaCSeL worst cases; the corpus is committed to
  git but excluded from the published crate, refreshed via
  `tools/sync-worst-cases.sh`). Under round-to-nearest, only distance to a
  **midpoint** makes an input hard — random sampling can never certify CR at
  the TMD points, which is why these published corpora are the gate.
- A **frozen-corpus regression guard** where CORE-MATH has no oracle (gamma,
  `log(x, base)`): `examples/gen_f64_*_cases.rs` (feature `mpfr`) scans ~800M
  inputs, keeps near-midpoint results with their MPFR answers, and the default
  test replays them oracle-free.
- **`test_<fn>_vs_mpfr`** (`#[cfg(feature = "mpfr")]`) — an independent
  broad sweep against MPFR ground truth, guarding against a bug shared with
  CORE-MATH. Run with `cargo test --release --features mpfr`.

Shared helpers live in `tests/all/common.rs` (`test_worst_univariate`,
`test_worst_bivariate`, `mpfr_sweep_univariate`, `parse_case_file`, …).

**`f128` (binary128): no proof-grade corpus exists anywhere — CORE-MATH
included — so a `q` function needs nothing from upstream but the perf headline.**
The binary64 `.wc` files are true worst cases from exhaustive searches; the
binary128 ones (`~/src/core-math-sys/vendor/src/binary128/<fn>/`) are
*structural*, and their oracle is MPFR (`<fn>q_mpfr.c`), exactly as ours is:

- `expq.wc` is threshold inputs plus "some worst cases computed with BaCSeL,
  with 10 to 56 identical bits after the round bit"; `expm1q.wc` is BaCSeL at
  `-m 10..59`. On a 2¹²⁸ domain, 56 bits is regression fodder, not a certificate.
- `sin.wc` / `cos.wc` (16k lines each, **no C yet**) come from `sin.sage`: per
  binade, the continued-fraction convergents of `2^(e−113)/(π/2)`, keeping the
  input nearest an odd (sin) or even (cos) multiple of π/2. The worst has **133
  identical bits** after the round bit — number theory beats BaCSeL, and
  CORE-MATH publishes the corpus *before* the implementation.
- `atan2q.sage` is the inverse trick (ARITH 2022 § IIB): pick a hard *output*
  `z` at 114 bits, take the convergents of `tan z` with 113-bit numerator and
  denominator, and `(y, x)` is hard by construction.
- The roots' corpora are labelled sections: special values, exact values, near
  powers of 2, subnormals, random.

The precedent is asinq/acosq/atanq before core-math 1.3.0 bound them
(`9b50018`, `7b4a074`; the switch-over is `0f1c593`). Ship a `q` function
CORE-MATH lacks in five steps:

1. **Oracle: MPFR at precision 113** — `metallic::f128_mpfr::cr_unop` /
   `cr_binop` (ternary-aware subnormalization) under `--features "f128 mpfr"`.
   Done when `test_<fn>q` sweeps mantissa-uniform inputs over every binade
   *plus* the function's own danger windows (`tests/all/f128_/asinq.rs` at `9b50018`:
   `breakpoints()`, `near_one()`) and `test_<fn>q_special` pins every edge.
2. **Corpus: generate it, with answers.** `tests/cases/<fn>q.wc` in the
   `f64_tgamma.wc` shape — `x, f(x)` per line, generator command in the
   header, an unconditional parser-count guard — so the strict gate replays
   under plain `--features f128` with no MPFR in CI. Done when all three layers
   are in:
   - *Derived from the math, not from search:* the exact and midpoint set
     (Lindemann–Weierstrass / Baker / Niven leave the transcendental families
     only the trivial ones — `exp 0`, `log 1`, `log2 2ᵏ`, `sinpi ⅙` — while the
     algebraic ones — roots, `hypot`, `pow`, `compound` — have real families and
     need exact-case *detection*, as f64 `pow` has), subnormal edges,
     overflow/underflow thresholds, and the `f(x) = x` boundary of every
     small-argument leg.
   - *The function's Diophantine family*, a ~30-line mpmath script
     `tools/gen_<fn>_f128_cases.py` mirroring the `.sage` scripts above:
     convergents of π/2 per binade for sin/cos; convergents of `tan z` for
     atan/atan2; near-Pythagorean triples for hypot; for a small-argument leg
     `x + c·xᵏ`, the inputs whose correction lands nearest a half-ulp of `x`.
   - *Regression layer:* an MPFR near-midpoint scan in the
     `examples/gen_f64_tgamma_cases.rs` mould (~30 bits past the round bit at
     10⁹ samples), or a BaCSeL run if matching their 56-bit layer matters.
3. **Ziv-gate soundness stays the rigorous part** — the in-source
   `ziv_soundness` certification below, unchanged. No corpus ever certified a
   fast leg.
4. **Accurate-leg precision by policy, not by a known worst case.** With
   relative error 2⁻ᴾ, Ziv's heuristic puts the expected number of misrounded
   inputs over the whole domain at ~2^(242−P). The analytic families add a
   *floor* that BaCSeL's 56 bits never reveals:

   | leg | precision | expected misses |
   |---|---|---|
   | `logq` accurate | 2⁻³⁴² | ~2⁻¹⁰⁰ |
   | `atan2q` accurate | 2⁻³⁸¹ | ~2⁻¹³⁹ |
   | floor for a `sinq` | ≥ 2⁻²⁴⁷ (113 + 133 + 1) | the π/2 family reaches every binade to 2¹⁶³⁸⁴, so Payne–Hanek must resolve δ to ~2⁻²⁵⁰ there |

5. **When upstream binds it** (`0f1c593` is the template): bump the `core-math`
   pin, switch `test_<fn>q_worst_cases` to bit-exact `core_math::<fn>q`, keep
   `test_<fn>q_vs_mpfr`, add the name to `FUNCS128` in
   `tools/sync-worst-cases.sh` (upstream files the trig corpora as `sin.wc`,
   not `sinq.wc`), keep our corpus beside theirs, then bench for the same-run
   ratio — the one deliverable that genuinely waits.

Candidates, in the order the structure favours: `sinq`/`cosq` (corpus already
upstream, unbindable for at least a release); `log2q`/`log10q`/`log1pq` on the
`logq` engine and the hyperbolics on `expq`/`logq`; `powq`, whose real risk is
the exact/midpoint tier, not the corpus; `erf`/gamma last — MPFR-only for good,
like their f64 versions.

## The CR mechanism (the house template)

Every f64 function follows the same shape:

1. a **lean fast leg** returning an (often un-normalized) `DoubleDouble` plus
   its error bound (the Ziv gate) — see [performance.md](performance.md) for
   how gate width/shape drive speed;
2. on gate failure, a **dd accurate tier** (~2⁻¹⁰⁵…2⁻¹⁰⁷), sometimes a third
   tier (triple-double, 128-bit `Dint`, 256-bit `Qint`);
3. a **sound final rounding**; and
4. for residual sub-2⁻¹⁰⁷ TMD cases, a small **exception database**
   (`EXP_HARD`, `ERFC_HARD`, …) holding CORE-MATH's analytic hard cases plus
   metallic's own corpus residuals.

**Sound-rounding pitfalls** (these caused every historical miss):

- A two-word dd's naive `high + low` **cannot** correctly round an exact
  half-ulp tie — RN ties-to-even discards the sub-½ulp sticky information.
- Round-to-odd applied to a two-word dd is **unsound** (the sticky sign is
  guessed from the renormalization residual). Sound options: a genuine third
  word (triple-double / `Dint`) so round-to-odd sees the true sticky, or an
  exception table.
- Use the crate's sound collapsers: `double::round_general64`,
  `round_general_signed64`, and `round_anchored(x, c)` (result-anchored
  small-argument legs, where the correction `c = f(x) − x` is computed as a dd
  and the exact `x` is added last).
- Overflow/underflow guards must test the **exact first-overflowing bit
  pattern** with the right comparator — five edge bugs (cosh/sinh/exp10/
  lgamma/tgamma) came from `>` vs `>=` on a rounded threshold (`3691a69`).

**The hard tier: port CORE-MATH's own accurate path.** When a function's ties
run past what dd or `Dint` can resolve, the reliable route to zero misses is
porting CORE-MATH's vendored accurate path from
`~/src/core-math-sys/vendor/src/binary64/<fn>/` (this is how atan2, pow, and
lgamma closed: 192-bit Tint, 256-bit qint, and triple-double respectively);
functions with milder ties are precision-lifts of the existing structure.
Note pow's extra lesson: ~3k of its misses were **exact rational-exponent
midpoints** that no amount of precision resolves — they need exact-case
*detection* logic.

## Ziv-gate soundness certification (non-negotiable)

A Ziv gate must exceed its leg's true error **with margin (≥ 2×)** — otherwise
a confident `lo == hi` can certify a value on the wrong side of a rounding
boundary, and no test short of the exact bad input will catch it (three latent
unsound gates were found in already-green code). The discipline:

- Every fast leg has an in-source `#[cfg(all(test, feature = "mpfr"))]
  mod ziv_soundness` proof: sample the leg's band (millions of points, 250-bit
  MPFR reference), compute worst `|err| / gate`, and assert it is `< 0.5`.
  The `worst_ratio` helper pattern lives in `src/f64_/log.rs` (search
  `mod ziv_soundness`); all 24 legs are certified.
- **A new or changed fast leg or gate ships its soundness proof in the same
  commit**, with the printed margin recorded in the commit message.
- Expose the leg's raw pair (e.g. `ln_fast_raw`, `log1p_wide_eval`) so the
  proof measures exactly what the gate sees.
- The tightest margin in the crate is log1p's wide leg at 2.25× — treat any
  change near it with suspicion.

**Watch the feature flag.** Run `cargo test`, **never `cargo test --all-features`**:
the `_no_fma` feature disables FMA usage, so under `--all-features` the suite would
exercise a different code path than the default build and produce misleading results
(per `CLAUDE.md`).

## Worst cases for bivariate (and complex) functions

When a function has **no published worst-case table**, you must find the
hard-to-round (HR) inputs yourself — but first check whether you need to. CORE-MATH
already publishes HR databases for the binary32 functions it covers, **including the
bivariate `powf`, `hypot`, and `atan2`**; for those, use the published cases (via
the `core-math` crate). The gap is functions outside that set — metallic's
two-argument **`log(x, base)`** (`= ln x / ln base`, neither a C99 function nor in
CORE-MATH), and the real/imag components of a future complex function.

A **naive exhaustive MPFR scan** — what metallic's own `gen_f32_log_cases` /
`gen_f64_log_cases` do (scan inputs, keep those landing within a normalized
threshold of a midpoint, freeze the survivors with their correct answers) — is fine
as a **regression-guard corpus**, and is the only practical option when the kernel
is already a double-double ≈2⁻⁹⁴ (true HR cases are then astronomically rare and a
margin-padded near-midpoint scan suffices). It does **not** enumerate the true worst
cases: the CORE-MATH paper measures this approach at ~10⁴ core-years for the full
`powf` grid and rejects it.

For true worst cases there is **no general clever bivariate algorithm**; the
practical move is to **fix one argument to make the problem univariate**, then run a
fast per-slice scan (CORE-MATH's difference-table Algorithm 1) or a
function-specific number-theoretic shortcut (almost-Pythagorean triples for `hypot`,
continued-fraction convergents of `tan z` for `atan2`). The mechanics are in the
CORE-MATH paper §II (Sibidanov–Zimmermann–Glondu, ARITH 2022; local copy
`~/doc/core-math-final.pdf`). Reach for **BaCSeL** (one- and two-variable HR search)
or the SLZ / Lefèvre univariate algorithms before writing a bespoke scanner.

**Complex functions** are a pair of real bivariate functions of `(re, im)` — e.g.
`clog`'s real part is `½·log(x²+y²)` = `log(hypot)`, its imag part is `atan2(y, x)`.
Correct rounding is per-component, and since `hypot`/`atan2` are CORE-MATH functions
with published HR tables, complex log/abs/arg can reuse those directly.

## Tooling

- `core-math` crate (CORE-MATH) — the primary oracle and reference.
- `rug` — MPFR-backed arbitrary-precision ground truth for f64.
- **BaCSeL** <https://gitlab.inria.fr/zimmerma/bacsel> — hard-to-round search for
  one- and two-variable functions when no published table exists (see the
  bivariate worst-case section above before scanning by brute force).
- Gappa <https://gappa.gitlabpages.inria.fr/> — machine-checked proofs of the
  kernel's rounding-error bound (the ε that Ziv's strategy and the binary64
  argument both depend on).
- Sollya `supnorm` — rigorous bound on the approximation error feeding that proof
  (see [coefficients.md](coefficients.md)).
- `criterion` (`cargo bench`) — confirm a correctly-rounded kernel stays competitive
  with core-math / std / libm.
