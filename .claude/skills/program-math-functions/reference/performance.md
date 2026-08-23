# Performance: the optimization playbook

Every f64 function is correctly rounded (issue #6, closed 2026-06-14); the open
front is speed, tracked in **issue #5**. The target is the **CORE-MATH ratio**
(`metallic ns / core_math ns`): CORE-MATH is the only baseline with the same
≤ 0.5 ulp contract, so it is the headline number in every bench table, commit
message, and issue update. std/libm are faithful-only (< 1 ulp) and legitimately
do less work — "beats std" can still be well above CORE-MATH.

Structurally, a metallic function is `mean ≈ fast_leg + fallback_rate ×
accurate_tier`. Every lever below moves one of those three terms; diagnosing
*which term* is the bottleneck comes before any code change.

## The loop

1. **Bench.** `RUSTFLAGS=-Ctarget-cpu=x86-64-v3 cargo bench --bench <fn>`
   (the host default has **no FMA** — an unflagged bench measures the wrong
   code). Criterion names: `metallic::<fn>` vs `core_math::<fn>` (f32 carries
   the `f` suffix; `f64::<fn>` is std, `libm::<fn>` is libm). Results land in
   `target/criterion/metallic__<fn>/…`.
2. **Tabulate.** `python3 tools/bench_ratio.py median` prints the paired ratio
   table grouped slow (>1.15) / equal / fast (<0.95).
3. **Pick** the worst *honest* laggard — see the sampling gotchas below before
   trusting a ratio.
4. **Diagnose** with the tree below. Do not skip to a favorite idea.
5. **Falsify first.** Before building anything, run the cheapest experiment
   that could kill the hypothesis (gut the polynomial, stub the gate, compare a
   sibling function that isolates the suspected stage). Example: before
   building log2's native base-2 leg, `metallic::log` vs `core_math::log` was
   checked at 0.994 — proving the gap was the closing ×log₂e multiply, not the
   shared reduction.
6. **Implement**, then verify: `cargo fmt`, `cargo test`; if a fast leg or Ziv
   gate changed, its `ziv_soundness` proof runs (and ships) in the same commit
   — see [correct-rounding.md](correct-rounding.md) § Ziv-gate soundness.
7. **A/B on the same box, same run.** Keep the change only if the win exceeds
   the noise floor (below). `git stash push src/f64_/<file>.rs` flips between
   arms cleanly.
8. **Commit** with the before→after ns and CORE-MATH ratio in the message.

## Benchmark hygiene (read before trusting any number)

- **Same-run ratios only.** CORE-MATH is a moving baseline: a `cargo update`
  once flipped asin from 0.81× (win) to 1.16× (loss) with zero metallic
  changes, because upstream rewrote their asin. Never compare a metallic time
  from today against a CORE-MATH time from last week.
- **Check the box before ranking anything.** `cat /proc/loadavg` AND
  `ps -eo pcpu,pmem,comm --sort=-pmem | head` — a single job using 30% of RAM
  at load "only" 2 inflated *every* bench 8–20% via memory bandwidth. Load
  average alone is not sufficient. Within-run metallic/CORE-MATH ratios stay
  valid under load; absolute ns and cross-run deltas do not.
- **Noise floor: ±1–3% on identical binaries**, even at moderate load. An A/B
  needs a calm box or a > 3% effect; a 2% "win" from one run is noise.
- **Run benches strictly serially.** "Contention hits both arms equally" is
  false on the 8-physical-core box: an 8-way concurrent f32 sweep (2026-07-11)
  inflated absolute times up to 2× and corrupted *within-pair* ratios — atanhf
  read 1.75× against a true same-code 1.02×, while other pairs skewed fast
  (expm1f 0.64 vs true 1.06).  Hyperthread-sibling port contention hits a
  table-heavy arm and a poly-heavy arm very differently.  One bench at a time;
  a full 42-target f32 sweep costs only ~12 minutes.
- **Sampling gotchas.** The `..` input range is representation-uniform, so
  about half of all f64 bit patterns are tiny (|x| < 2⁻²⁶) and a tiny-arg fast
  return can swallow the bench (f64 atan: ~96% of `..` draws skipped the
  kernel). Use `bench::Exponents(lo..=hi)` (log-uniform magnitudes) for
  magnitude-gated kernels, `PositiveExponents` for positive-only domains, and
  temporarily repoint a bench range (e.g. `-2.0..=2.0`) to isolate one band —
  the per-band split benches like `metallic::tgamma_reflect` exist for this.
- **Perf claims need the disassembly**, not intuition, when the hypothesis is
  about branches or instruction selection: build a probe example whose
  `#[inline(never)]` wrapper calls the function through a `black_box`'d fn
  pointer, `objdump -d` it, and grep for `jbe`/`jae` (branches) vs `cmov`.

## Diagnosis tree (in priority order)

**1. Is there a data-dependent sign/parity/magnitude branch in the hot leg?**
Grep for this FIRST on any random-input laggard. Criterion draws fresh random
input every iteration, so a 50/50 branch mispredicts half the time — a ~3–5 ns
tax that can be >10% of the function. Fixes, cheapest first:

- Sign/parity flips are the cheapest arms of all: compute the **positive
  magnitude**, run the Ziv gate on it (no `.abs()` needed), and XOR the sign
  bit onto the certified scalar at the end — exact, and commutes with
  round-to-nearest. (f64 tgamma reflection, commit `ffb264b`: 1.15→1.03.)
- Otherwise compute both arms unconditionally and let LLVM `cmov`-select —
  worth it when the redundant arm is cheap relative to a mispredict.
- LLVM's cost model is finicky; verify with the disasm probe. Known traps:
  ending an arm in `copysign` can make LLVM branch and sink a `sqrt` into one
  side (an explicit `if x.is_sign_positive() { a } else { b }` produced the
  `cmov`); `TABLE[usize::from(cond)]` can lower back into a branch (branchless
  arithmetic like `π/2 − copysign(π/2, x)` is safer); a checked `as` cast
  emits a saturating compare+cmov chain — use `to_int_unchecked` where the
  range is already proven, with a `// SAFETY:` comment.

**2. Fallback-bound or fast-leg-bound?** Estimate the Ziv fallback rate before
touching either tier: `fallback ≈ 2 · gate / ulp(result)`. Measure it directly
with a temporary `static AtomicU64` counter in the gate plus a dense sweep
test, or bench a forced-fallback stub (make the gate always fail). Instrument,
measure, revert.

- **Tier-trim leverage rule**: cheapening the accurate tier only moves the mean
  when the fallback rate is non-trivial. Measured rates: trig ~2.4%, cosh/sinh
  ~0.2%, exp and erf families sub-1% — all fast-leg-bound, so accurate-tier
  work there is sub-noise. Only gamma-style functions (fallback spikes near
  zeros/poles) reward tier work.
- If fallback-bound, the levers are gate **width and shape**, a cheaper middle
  tier, or a better leg — see the pattern catalog.
- The gate's floor is *soundness* — it can never be loosened below the leg's
  certified error, so a leg rewrite is sometimes the only way to shrink it.

**3. Latency-, throughput-, or port-bound?** (when the leg itself is slow)

- **Latency-bound**: a serial dependency chain longer than the out-of-order
  window. Symptom: the function is much slower than its µop count suggests.
  Fix by restructuring onto a shorter-chain shape and using the ordered folds
  (`add_ordered`) instead of full renormalizations. Exemplar: f64 tanh
  (`592b3a6`) — the mid-band `E/(E+2)` combine was ~150 serial cycles; the
  algebraically equal `1 − 2/(e²ˣ+1)` shape is ~50, taking tanh from 1.2× to
  0.94× (beats CORE-MATH).
- **Throughput-bound**: µops saturate the pipeline. Symptom: deleting a couple
  of flops changes nothing. The exp family is here (~35 µops @ ~9 ns) —
  micro-cuts under ~2% are invisible; only a structurally smaller algorithm
  (e.g. the two-level 2^(j/4096) table that created the lean leg) moves it.
- **Port-bound**: one execution port saturated. `Dint::mul` is the example
  (~6.6 ns; three u128 multiplies fill the multiplier ports), so Estrin-izing
  a `Dint` polynomial buys ~nothing — the only `Dint` lever is **fewer terms**
  (minimax instead of Taylor).

**4. Suspect the table/polynomial? Gut it first.** Truncate the polynomial to
2 terms (or stub the table) and re-bench. If the time doesn't move, the
polynomial is free and a finer table won't help — the cost is elsewhere
(atan: the DD divisions dominate; tail truncation moved it 0.0 ns).

**5. Consumer overhead.** Once shared legs (exp, ln) are at parity, a slow
consumer (atanh, asinh, lgamma, …) usually carries its own removable dd op on
the critical path that the round-to-nearest-only budget lets you drop (a
renormalize, a low-word fold, a division). Judge candidates by **loop-carried
latency** and whether the op **overlaps** other work (ILP) — an op that runs
in the shadow of a division is already free, and removing it buys nothing
(the asinh sqrt/div-overlap dead-end below).

## Pattern catalog (proven winners, with exemplars)

- **Table-driven fast leg** — the go-to for a fast correctly-rounded function
  that is smooth over a bounded range: a LUT of cells, each one cache line
  (`#[repr(C, align(64))]`, dd lead coefficients + f64 tail), per-cell
  **Chebyshev** fits (beat Taylor's edge truncation — one fewer coefficient),
  a shared straight-line tail, Ziv-gated against the *existing, untouched*
  accurate path. Exemplars: erf `ERF_TABLE`, asin/acos `ASIN_CELLS`
  (`747765e`), gamma `GammaCell`. Index with `to_int_unchecked`.
- **Gate width AND shape set the fallback rate.** A tighter *leg* does not cut
  fallback by itself — the gate does (`≈ 2·gate/ulp`). Profile the leg's true
  error per sub-band with MPFR and shape the gate to hug it: lgamma's Stirling
  gate went from flat 2⁻⁵⁶ to `A/t + (t−½)·2⁻⁶⁶` (`cc29e52`) because its
  error *shrinks* with z — copying tgamma's z-growing gate there was unsound.
  Correction-scaled gates `eps = |z|·(C·|r|+F)+G` (asin/acos) and per-region
  gates (lgamma near its zeros) are the same idea. Have the fast leg **return
  its own gate** so regions can differ.
- **Two-tier fallback**: fast leg → *cheap* dd refinement (~50–70 ns) → the
  heavyweight tier (triple-double/`Dint`, ~10³ ns) only on genuine TMD misses.
  Wiring the heavy tier directly onto first-gate misses costs +12 ns on the
  mean (acos episode); `#[cold]`/`#[inline(never)]` does not save you.
- **Exact-z lattice reduction** (`tools/gen_ln_exact_f64.py`): choose per-cell
  reciprocals `r` on a coarse grid so `z = fma(r, m, −1)` is *exact* in one
  f64 — exactness without Gappa: if `r` is a multiple of 2⁻ᵍ and `m` of 2⁻⁵²,
  `z` is exact iff `|z| ≤ 2^(53−g−52)`; `g ≤ k+1` (k index bits) guarantees a
  grid point per cell. This deleted ln's double-double reduction (`4bf01e9`);
  the same geometry gave log2 a native base-2 leg where `e + l1` is a plain
  exact add (`10f8830`).
- **Consumer-gate audit**: whenever a shared leg's error bound loosens, audit
  *every* consumer's gate margin in the same change (the exact-z ln rewrite
  forced `LGAMMA_FAST_BOUND` 1024→256 and cut pow's margin 8×→2.8×).
- **Mixed-precision legs**: the accurate tier guarantees correctness, so the
  fast leg's precision is a *perf* dial. Put the polynomial's high-degree tail
  in plain f64 (`poly_dd_split`, SPLIT=4) — erf/erfc 2.1× with a measured
  fallback rate identical to all-dd.
- **4-lane recurrence parallelization**: a serial dd-multiply recurrence
  (gamma) re-associates onto 4 independent lanes; build each factor *exactly*
  (`from_sum(z, k)`) so low bits survive; reassociation costs only ~2⁻¹⁰⁰.
- **Ordered folds + fused divisions**: `DoubleDouble::add_ordered` (Fast2Sum
  highs, plain low carry, no renormalize — requires `|a.high| ≥ |b.high|`,
  which each caller proves in a per-module `#[cfg(test)] mod fold_ordering`)
  and `add_loose` (2Sum highs, for the rare unordered fold) shorten a
  Ziv-gated leg's serial chain (`058508f`, `171638c`, `740eb07`). For
  quotients, one fused hardware division (see `two_over` in `src/f64_/hyp.rs`)
  beats `recip()` + `Mul`.
- **Fuse reflections to save divisions**: rewrite `u = (1/a − c)/(1 + c/a)` as
  `u = (1 − a·c)/(a + c)` — never form `1/a` as a dd. atan 3→2 divisions
  (`7e942e7`), atan2 4→2 (`622010a`).
- **In `u128` fixed point, the shifts cost more than the multiplies.** LLVM
  has no 128-bit funnel shift: `(a >> k) | (b << 1 << (127 - k))` with a
  variable `k` becomes a `shrd`/`shrx` pair plus a `cmovne` per limb.  Cut the
  window out of 64-bit limbs instead — a `cmov` chain to pick the limbs, then
  one `shrd` each (binary128 `frame`, `b611a8f`: 1.18x -> 1.13x).  The same
  applies to the *rounding* tail: if the discarded width is constant for all
  normal results, split the variable-shift cases into a `#[cold]` function and
  the masks become literals (`964583a`: 1.13x -> 0.96x, the single biggest win
  of that campaign).  Ablate before redesigning a kernel — for binary128 exp,
  deleting a whole 128x128 multiply moved **nothing**, while deleting the
  rounding masks moved 5 ns.  But read an ablation as bounding count *plus*
  latency, not latency alone: stubbing binary128 atan2q's polynomial priced it
  at 20 ns, yet halving its serial depth returned only 2.4 — back-to-back
  bench iterations already overlapped the chain.
- **Binary128 bivariate cost centers** (atan2q campaign, 1.58x -> 1.24x):
  narrowing the whole fast pipeline from two `u128` limbs to one — reciprocal
  top limb, quotient, polynomial, final product — bought the bulk; 3-mul
  approximate high products (`mhi_approx`, ≤ 2 units short) and mask-select
  add-or-sub over 50/50 quadrant branches the rest.  The open lead for the
  remaining gap: CORE-MATH seeds its reciprocal with a 63-bit hardware divide
  and needs one Newton step where an `f64` seed (51 bits) forces two.
- **The real round-to-nearest dividends** (vs CORE-MATH's 4-mode burden):
  un-normalized dd returns consumed directly by the Ziv gate, and free FMA
  contraction in `crate::poly` (CORE-MATH's `FENV_ACCESS ON` inhibits it).
  NOT `fast_ldexp` tricks or branch micro-hacks — those were measured dead.

## Falsified dead-ends — do not retry these

Each was implemented, measured, and reverted. Re-proposing one without new
evidence wastes a session.

| Idea | Verdict |
| --- | --- |
| `DoubleDouble::Mul` "3-deep parallel" restructure | Net regression (tgamma +28%): judge dd primitives by **loop-carried latency through `self.low`** in Horner/recurrence chains, not isolated depth. Current serial form carries `low` through 1 op/step; the "parallel" form carries 3. |
| exp2 base-2 coefficient bake | Correct but invisible (p=0.13) — exp family is throughput-bound; ~2-flop cuts don't show. |
| asinh sqrt/div overlap scheduling | 1.094→1.094 *exactly*; the chain feeds only `arg.low`, and ln's high side dominates. The remaining asinh gap is structural. |
| atan finer table / poly work | Gutting the tail moved 0.0 ns — divisions and dd ops are the cost. |
| unsafe SIMD `fast_ldexp` | Within noise; hidden by out-of-order execution. |
| Bit-trick edge checks (replace FP compares) | No gain — a predicted-not-taken `comisd` is ~free. |
| Estrin on `poly_dint` | ~2% on the forced path — `Dint::mul` is port-bound; fewer terms is the only lever. |
| Cutting multiplies in the binary128 exp kernel | Removing one of the three `mul127`s in the table product moved 0.0 ns. The kernel is latency- and *shift*-bound, not multiplier-bound; CORE-MATH's cheaper `u64 x u128` Horner has the same chain depth, so porting its shape is unlikely to pay. |
| Series fast legs for **wide** bands | atanh +34% (75% fallback at band edge). A plain-f64 series leg floors at ~2⁻⁵³ relative on the correction, so it only pays when the band is narrow (asinh: win) or the general path is heavier than a polynomial. |

## Current standings

Live status (ratio table, open laggards, per-function notes) is maintained in
**issue #5** — read it with `gh api repos/jdh8/metallic-rs/issues/5 --jq
.body` and its comments before picking a target. As of 2026-07-02 (calm box):
no function above 1.15×, and the remaining ~1.05–1.10 cluster (asinh, asin,
atanh, log1p, exp2/exp10) has no known mechanism — treat those as research,
not backlog.
