# Correct rounding

**The target is round-to-nearest, ties-to-even:** compute the exact mathematical
result and round once to the destination format. A rounded display of an error
as `0.5 ulp` does not establish that result or its tie handling. Faithful rounding
returns one of the representable neighbors bracketing the exact result; correct
rounding chooses the required one. Treat a known misrounding as a bug, and label
evidence as sampled testing, exhaustive verification, or a full-domain proof.

**Baseline: strict regression gates are active.** Issue #6 closed on 2026-06-14;
a red gate in `cargo test` is a regression to fix. This is not a blanket proof
of every implementation. Preserve both the tests and any established error
bounds when changing code; see [performance.md](performance.md).

## The Table Maker's Dilemma

To return the correctly-rounded `f(x)` you must decide which side of a rounding
boundary the *exact* `f(x)` lies on. Under round-to-nearest the boundary is the
midpoint between two representables. Results can be extremely close to a midpoint.
After exact boundary cases are handled, a finite input domain has a finite
sufficient precision, but **an empirically chosen precision is not a proven
bound**. Finding or avoiding the need to know that bound is the dilemma.
The "hardness" of an input is the length
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
   level needs a *correct* ε — prove it with Gappa or rigorous interval/error
   analysis, don't guess. The last level must also certify its result: either
   prove its precision sufficient, or keep refining certified intervals until
   both endpoints round identically. Handle exact boundary cases separately and
   justify convergence and termination. An unbounded-arithmetic termination
   argument does not establish a practical memory or integer-counter bound.

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

- **CORE-MATH** <https://core-math.gitlabpages.inria.fr/> (MIT), through the
  **`core-math` crate** — a test and design reference, with hard-to-round
  databases. Check proof and database coverage per function and format. In
  particular, agreement with a related binary128 transcendental implementation
  does not establish correct rounding for either implementation.
- **`rug` / MPFR** — use a correctly rounded operation directly at the target
  precision, with IEEE exponent limits and ternary-aware subnormalization, or
  refine rigorous enclosures until their target-format roundings agree. Merely
  computing at a fixed higher precision and rounding again can double-round.

## How you *prove* a function is correctly rounded

**Unary `f32` (binary32): exhaustive verification is feasible.** Only 2³² bit
patterns exist. Each f32 function has a per-function test file (`tests/all/f32_/sinf.rs`,
`tests/all/f32_/expf.rs`, …) whose core is one line: `common::test_all_f32(metallic::sinf,
core_math::sinf)` — it loops **every** `f32` and compares against the `core-math`
oracle via the `Identity` trait in `tests/all/common.rs` (bit-equal, NaNs equal).
A completed clean sweep establishes agreement for that build, conditional on
the oracle and harness being correct. Record the target, features and oracle
revision; it does not cover every compiler or FMA configuration automatically.
Bivariate functions use `common::test_bivariate_cases`, which does not exhaust
the 2⁶⁴ input pairs;
`common::truncate_errors` caps the report at 250 mismatches so a regression fails
fast.

**`f64` (binary64): cannot brute-force** (2⁶⁴ inputs) — so the repo reproduces
CORE-MATH's own per-function check discipline. Each `tests/all/f64_/<fn>.rs` carries:

- **`test_<fn>_worst_cases`** — the strict correct-rounding gate: bit-exact vs
  the `core-math` oracle on CORE-MATH's hard-to-round corpus
  `tests/cases/<fn>.wc` (their BaCSeL worst cases; the corpus is committed to
  git but excluded from the published crate, refreshed via
  `tools/sync-worst-cases.sh`). Under round-to-nearest, only distance to a
  **midpoint** makes an input hard. To turn these checks into a proof, establish
  that the corpus includes every case unresolved by this implementation's
  rigorous error bounds, then verify those cases and all special paths.
  Published coverage varies by function; see
  [CORE-MATH's worst-case status](https://core-math.gitlabpages.inria.fr/worst.html).
- A **frozen-corpus regression guard** where CORE-MATH has no oracle (gamma,
  `log(x, base)`): `examples/gen_f64_*_cases.rs` (feature `mpfr`) scans ~800M
  inputs, keeps near-midpoint results with their MPFR answers, and the default
  test replays them oracle-free.
- **`test_<fn>_vs_mpfr`** (`#[cfg(feature = "mpfr")]`) — an independent
  broad sweep against MPFR ground truth, guarding against a bug shared with
  CORE-MATH. Run with `cargo test --release --features mpfr`.

Shared helpers live in `tests/all/common.rs` (`test_worst_univariate`,
`test_worst_bivariate`, `mpfr_sweep_univariate`, `parse_case_file`, …).

**`f128` (binary128): regression evidence is not a full-domain proof.** The
available transcendental corpora contain constructed near-boundary cases,
partial searches and samples. They do not establish a sufficient precision for
every input of these implementations. Roots and other algebraic operations can
instead use exact integer comparisons; their proofs must cover the complete
implementation, including any shortcuts. Do not infer their status from the
transcendental hard-case problem.

For a new or changed `q` function:

1. **Use an independent oracle.** `metallic::f128_mpfr::cr_unop` / `cr_binop`
   operate at precision 113 with ternary-aware subnormalization under
   `--features "f128 mpfr"`. Sweep mantissa-uniform inputs across exponent
   bands and the function's danger windows; pin special cases separately.
   Record counts, seeds, build configuration and oracle version.
2. **Keep generated corpora with answers.** Preserve the generator command and
   an unconditional parser-count guard. Include exact cases, midpoint cases
   where possible, subnormal and overflow boundaries, reduction seams,
   function-specific Diophantine constructions, and near-midpoint scans.
   Prove any claimed exhaustive classification separately. For example,
   `exp(0)`, `log(1)`, `log2(2^k)` and `sinpi(1/2)` are exact cases; roots,
   powers and `compound` need their own exact-boundary analysis. Do not assume
   a theorem for one family settles another, especially gamma or error functions.
3. **Prove every accepting gate.** Derive a uniform bound for reduction,
   constants, approximation, arithmetic and reconstruction. Keep the in-source
   `ziv_soundness` sampled checks as regression tests of that argument, not as
   substitutes for it.
4. **Justify the final tier.** Either prove its precision sufficient over the
   remaining domain, or add adaptive certified enclosures that refine until both
   endpoints round identically. Handle exact boundaries, prove convergence and
   termination, and distinguish the mathematical algorithm from the finite
   implementation's memory and counter limits. A wide fixed fallback without
   this argument remains an empirically tested approximation. Its limb count
   and heuristic expected-miss estimates are not correctness guarantees.
5. **Retain independent checks when upstream binds it.** Add the pinned
   `core_math::<fn>q` comparison and upstream corpus, keep the MPFR tests and
   local corpus under a distinct name, and benchmark the same-run ratio.
   Shared code structure makes independent verification especially valuable.

The binary64 proof method transfers: rigorous local error bounds plus enough
hard-case coverage to settle the last tier. Its search cost does not transfer
automatically: one binary128 binade has 2^112 significands versus 2^52 for
binary64. Adaptive interval refinement can avoid a complete hard-case list
for suitable functions, but requires the function-specific arguments above.

Useful primary references:

- [de Dinechin, Lauter and Muller (2007), correctly rounded logarithms](https://www.numdam.org/article/ITA_2007__41_1_85_0.pdf):
  implementation error bounds and a complete rounding argument.
- [Lefèvre, Ly and Zimmermann (2026), binary64 sin/cos/tan hard cases](https://www.arith2026.org/papers/Computing%20hard-to-round%20cases%20of%20sin_2C%20cos_2C%20tan%20in%20double%20precisio.pdf):
  exhaustive hard-case search for those functions and that format.
- [Ziv (1991), adaptive precision](https://research.ibm.com/publications/fast-evaluation-of-elementary-mathematical-functions-with-correctly-rounded-last-bit):
  refine approximations and error bounds until the rounding decision is settled.

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
- A **rounded intermediate is not a point**: feeding an IEEE quotient (or
  any rounded value) into a Ziv gate as `{high: q, low: 0}` charges nothing
  for its own ½-ulp error, and the gate accepts blindly.  Issue #11:
  `atan2(2⁻⁵², 1 − 2⁻⁵³)` — the exact ratio sits 2⁻¹⁰⁶ past a midpoint, the
  division rounds up, `atan`'s `−q³/3` rounds down, and the "atan(q) ≈ q far
  below ½ ulp" shortcut returned the quotient.  Either carry the exact
  residual (`DoubleDouble::from_quotient`) under the real gate, or *prove*
  the shortcut against the midpoint grid (a ratio of two 53-bit significands
  keeps `> 2⁻¹⁰⁸·q` clear of every 54-bit midpoint and never lands on one —
  enough only when the dropped term is far below that, as in atan2's deep
  `q < 2⁻⁹⁵⁹` band).
- A **relative gate underflows**: `|v|·2⁻⁶³` is zero for `|v| < 2⁻⁹⁵⁹`, so a
  scale-invariant gate silently becomes a point gate near the subnormal
  floor.  Route that band to a tier whose rounding is proved outright.
- **Samplers with even LSBs**: `k/2⁵³` folded exactly into `[−1, 1]` by one
  `fma` is a multiple of 2⁻⁵² — every draw has a zero last bit, and inputs
  like `1 − 2⁻⁵³` are unreachable.  `tests/all/common::uniform` now adds 64
  hash bits with a *rounding* final add; build any new sampler the same way.

**The hard tier: port CORE-MATH's own accurate path.** When a function's ties
run past what dd or `Dint` can resolve, the reliable route to zero misses is
porting CORE-MATH's vendored accurate path from
`~/src/core-math-sys/vendor/src/binary64/<fn>/` (this is how atan2, pow, and
lgamma closed: 192-bit Tint, 256-bit qint, and triple-double respectively);
functions with milder ties are precision-lifts of the existing structure.
Note pow's extra lesson: ~3k of its misses were **exact rational-exponent
midpoints** that no amount of precision resolves — they need exact-case
*detection* logic.

## Rigorous Ziv bounds and sampled gate checks

A Ziv gate must enclose its leg's true error for every input on that path. Keep
an engineering margin (the sampled checks require > 2×), but do not mistake a
sampled margin for a uniform bound. Otherwise
a confident `lo == hi` can certify a value on the wrong side of a rounding
boundary, and no test short of the exact bad input will catch it (three latent
unsound gates were found in already-green code). The discipline:

- Every fast leg has an in-source `#[cfg(all(test, feature = "mpfr"))]
  mod ziv_soundness` regression check: sample the leg's band (millions of
  points, 250-bit MPFR reference), compute sampled worst `|err| / gate`, and
  assert it is `< 0.5`. Account for reference error if using the measurement
  quantitatively; fixed-precision MPFR is not an exact real value.
  The `worst_ratio` helper pattern lives in `src/f64_/log.rs` (search
  `mod ziv_soundness`). Passing this test does not certify the unsampled domain.
- **A new or changed fast leg or gate ships its sampled check in the same
  commit**, with the observed margin recorded in the commit message. To claim
  a certified gate, also supply or update the rigorous error analysis covering
  the whole path, including reduction, constants and integer truncation.
- Expose the leg's raw pair (e.g. `ln_fast_raw`, `log1p_wide_eval`) so the
  test measures exactly what the gate sees.
- The recorded log1p wide-leg sample had 2.25× margin; that is an observed
  margin, not a proven lower bound over all inputs.

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

A **naive MPFR sample scan** — what metallic's own `gen_f32_log_cases` /
`gen_f64_log_cases` do (scan inputs, keep those landing within a normalized
threshold of a midpoint, freeze the survivors with their correct answers) — is fine
as a **regression-guard corpus**. Greater working precision does not make that
scan a completeness argument. It does **not** enumerate the true worst
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
Correct rounding is per-component. Published `hypot`/`atan2` cases are useful
regressions, but composing correctly rounded functions does not generally give a
correctly rounded composition. In particular, `log(hypot(x,y))` needs error
propagation or a direct enclosure of the exact expression.

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
