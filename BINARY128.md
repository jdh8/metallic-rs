# Binary128 (`f128`) functions

Metallic's binary128 functions use the libquadmath-compatible `q` suffix and
are gated behind a nightly-only feature:

```toml
[dependencies]
metallic = { version = "0.3.0", features = ["f128"] }
```

```rust,ignore
#![feature(f128)]

assert_eq!(metallic::sqrtq(4.0_f128), 2.0);
assert_eq!(metallic::rsqrtq(4.0_f128), 0.5);
assert_eq!(metallic::cbrtq(8.0_f128), 2.0);
assert_eq!(metallic::exp2q(10.0_f128), 1024.0);
assert_eq!(metallic::exp2m1q(10.0_f128), 1023.0);
assert_eq!(metallic::exp10m1q(3.0_f128), 999.0);
assert_eq!(metallic::logq(1.0_f128), 0.0);
assert_eq!(metallic::log2q(1024.0_f128), 10.0);
assert_eq!(metallic::log10q(1000.0_f128), 3.0);
assert_eq!(metallic::log1pq(1.0_f128), core::f128::consts::LN_2);
assert_eq!(metallic::log2p1q(3.0_f128), 2.0);
assert_eq!(metallic::log10p1q(99.0_f128), 2.0);
assert_eq!(metallic::powq(2.0_f128, 0.5), core::f128::consts::SQRT_2);
assert_eq!(metallic::sinpiq(0.5_f128), 1.0);
assert_eq!(metallic::cospiq(1.0_f128), -1.0);
assert_eq!(metallic::tanpiq(0.25_f128), 1.0);
assert_eq!(metallic::fmaq(3.0_f128, 4.0, 5.0), 17.0);
assert_eq!(metallic::frexpq(48.0_f128), (0.75, 6));
assert_eq!(metallic::ldexpq(3.0_f128, 4), 48.0);
```

Run its tests with `cargo +nightly test --features f128`.  Building the
[CORE-MATH] oracle for binary128 needs `CC=clang`: CORE-MATH's `hypotq.c`
calls `__builtin_addcl`, which GCC does not provide.  The f128 side runs on
x86-64 GNU/Linux only — CORE-MATH's `binary128/` sources need `__float128`,
which clang refuses on every Darwin target.

[CORE-MATH]: https://core-math.gitlabpages.inria.fr/

## Status

Each function is done when both gates hold:

- **CR** — correctly rounded, all strict gates green (bit-exact vs
  `core_math::<fn>q` on the worst-case corpus, deterministic samples, MPFR).
  `exp2m1q`/`exp10m1q`/
  `sinq`/`cosq`/`sincosq`/`tanq`/`sinpiq`/`cospiq`/`tanpiq`/`log2q`/`log10q`/
  `log1pq`/`log2p1q`/`log10p1q`/`powq` have no CORE-MATH binding yet:
  their strict gate replays a home-grown corpus that carries its
  MPFR answers, and the matching CORE-MATH `f64` functions cross-check
  them oracle-free.
- **Perf** — measured same-run median ratio `metallic::<fn>q / core_math::<fn>q`
  ≈ 1× or better on the recorded workload and hardware. Archived results are in
  [BENCHMARKS.md](BENCHMARKS.md); [ANALYSIS.md](ANALYSIS.md) helps diagnose costs.
  To time one function on your own machine run
  `RUSTFLAGS=-Ctarget-cpu=x86-64-v3 cargo +nightly bench --features f128
  --bench <fn>q`, then `python3 tools/bench_ratio.py median`.  Each bench also
  carries a `f128::` lane (nightly `std`) and, where CORE-MATH has no binding,
  a `quadmath::` one; both are faithful-only, so they are context rather than
  the headline — see [Baselines](#baselines).

| Function | CR |
|----------|:--:|
| `acosq`  | ✅ |
| `asinq`  | ✅ |
| `atanq`  | ✅ |
| `atan2q` | ✅ |
| `cbrtq`  | ✅ |
| `cosq`   | ✅ |
| `cospiq` | ✅ |
| `exp10q` | ✅ |
| `exp10m1q` | ✅ |
| `fmaq`   | ✅ |
| `frexpq` | ✅ |
| `exp2q`  | ✅ |
| `exp2m1q` | ✅ |
| `expm1q` | ✅ |
| `expq`   | ✅ |
| `hypotq` | ✅ |
| `ldexpq` | ✅ |
| `log2q`  | ✅ |
| `log2p1q` | ✅ |
| `log10q` | ✅ |
| `log10p1q` | ✅ |
| `log1pq` | ✅ |
| `logq`   | ✅ |
| `powq`   | ✅ |
| `roundq` | ✅ |
| `rsqrtq` | ✅ |
| `sincosq` | ✅ |
| `sinq`   | ✅ |
| `sinpiq` | ✅ |
| `sqrtq`  | ✅ |
| `tanq`   | ✅ |
| `tanpiq` | ✅ |

`sincosq` returns the pair off one reduction and one pair of series, so both
halves are the separately gated `sinq` and `cosq` bit for bit — that identity,
replayed over both corpora, is its gate; libquadmath binds it, `f128::` does
not.

`exp2m1q`, `exp10m1q`, `log2p1q`, `log10p1q`, `sinpiq`, `cospiq`, and
`tanpiq` have standalone benchmarks; neither CORE-MATH nor libquadmath
currently provides these entry points. They are not in the historical snapshots in [BENCHMARKS.md](BENCHMARKS.md).
The initial `exp2m1q` / `exp10m1q` x86-64-v3 traces at `x = 1.7` cost
169 / 183 llvm-mca cycles (2026-09-16); regenerate with
`python3 tools/analysis.py asm --only exp2m1q,exp10m1q --isa v3`, then
`python3 tools/analysis.py mca --only exp2m1q,exp10m1q`. Concurrent workloads
made the initial Criterion timings unsuitable for an absolute speed claim.

`fmaq`, `frexpq`, `ldexpq` and `roundq` are exact operations rather than
approximations, so neither column means what it does for the rest of the
table.  `frexpq`, `ldexpq` and `roundq` are bit manipulation, gated on their
defining invariants — the exponent-field identity, the subnormal ladder, the
exact round trip, an independent integer reconstruction — plus an MPFR sweep;
none has an interesting cost.  `fmaq` is the *platform's*
binary128 fused multiply-add, not metallic's: it is glibc's `fmaf128` wherever
`long double` is binary128, and on a target where it is narrower LLVM lowers it
to `fmal`, which computes the narrow function of the low half of each operand.
Keep it off any path that has to be right everywhere; see its rustdoc.

## Coverage gap

Fifteen `f64` entry points have no binary128 counterpart yet.  `fmaq`,
`frexpq`, `ldexpq`, `roundq` and `sincosq` are done — the first three were
written already, `roundq` is the few lines of bit manipulation its row
promised, and `sincosq` hoisted `trig::reduce` once across both legs as
planned.  For the rest, what follows is the plan, not a status report.

| to do | rides on |
|-------|----------|
| `asinpiq`, `acospiq` | `asin.rs` with `1/π` folded into `PHI` |
| `atanpiq`, `atan2piq` | `atan2.rs` with `1/π` folded into the tables |
| `sinhq`, `coshq`, `tanhq` | `exp.rs` both legs; `tanhq` also `tan.rs`'s `quotient`/`refine` |
| `asinhq`, `acoshq` | `log.rs` plus `roots::sqrt_wide_seeded` |
| `atanhq` | `log1pq`'s front end and `SMALL_GATE` band |
| `compoundq` | `pow.rs`'s three tiers with `log::one_plus` in front |
| `erfq`, `erfcq`, `tgammaq`, `lgammaq` | nothing yet — new tables, new generators |

`sqrtq` is the one entry point that goes the other way: `f64` defers to
`f64::sqrt`, while binary128 needs its own because no `f128::` method is an
oracle.

### What every one of them costs

`FUNCS128` in `tools/sync-worst-cases.sh` is `sqrt rsqrt cbrt hypot exp exp2
exp10 expm1 log asin acos atan atan2`.  **None of the remaining functions has a
CORE-MATH binding**, so none of them gets the cheap gate.  Each follows the
`sinq`/`powq` route from CLAUDE.md — an `examples/gen_f128_*_cases.rs`
generator producing a corpus that carries its own MPFR answers, the
`--features mpfr` sweep, the matching `f64` CORE-MATH function as the
oracle-free cross-check, and `mod ziv_soundness` in the same commit as the
fast leg it certifies.  Budget the generator at roughly half the work of the
function.

Benchmark baselines split three ways.  libquadmath binds the hyperbolics,
`erfq`, `lgammaq` and `tgammaq`; it has no entry point at all for the seven
π-scaled functions, `exp2m1q`, `exp10m1q` or `compoundq`, which therefore get
no external lane (as `log2p1q` and `log10p1q` already do not) and fall back to
`tools/analysis.py` cycles as the headline.

### Order

**Phase 0 — exports.**  ~~`fmaq`, `frexpq`, `ldexpq`, `roundq`, `sincosq`~~
(done).  `sincosq` was the only real work of the five, and the point was to
share one `reduce` rather than call `trig` twice: the pair costs about what
the sine alone does, against the two calls' sum.

**Phase 1 — `exp2m1q`, `exp10m1q` (done).** Both reuse the exponential
frame and carry their own base's Taylor ratio near zero; see below.

**Phase 2 — the π-scaled seven.** `sinpiq`, `cospiq`, and `tanpiq` are
implemented: `trigpi.rs` splits `n = round(256x)` and the residual exactly,
then scales the residual by π inside the existing `trig.rs`/`tan.rs` frames.
For the inverse four, folding `1/π` into the constants makes the quadrant offsets dyadic — `atan2piq` adds 0, ½, 1
instead of 0, π/2, π, so those additions become exact and an error term
vanishes; the tables become `atan(i/64)/π` and `asin(j/128)/π`, regenerated
with a `--pi` flag on the existing generators, still mathematical constants and
not fits.  The direct three return the exact integer, half-integer, and tangent
quarter-integer values explicitly, including IEEE signed-zero and pole parity.
Their corpora cover those seams and their neighbors through the all-integer
range above 2^112.

**Phase 3 — the hyperbolics.**  The bulk of the remaining line count.
`sinhq`/`coshq` are `(e^x ∓ e^-x)/2` on `exp.rs`'s fast leg with `expm1q`'s
small series covering `sinh`'s cancellation near zero; `f64`'s `hyp.rs`
(`combine`, `two_over`) is the structural template.  `tanhq` wants `tan.rs`'s
Newton quotient, not a soft-float divide.  `asinhq`/`acoshq` are
`log(x + √(x² ± 1))` with `x² ± 1` formed exactly — `log::one_plus` is that
primitive and `sqrt_wide_seeded` is the root.  `atanhq` is
`½·log1p(2x/(1 − x))` and is nearly free once `log1pq`'s small band is reused.

**Phase 4 — `compoundq`.**  `powq`'s engine with `log::one_plus` ahead of the
logarithm.  The `exact` tier needs its own analysis (`(1+x)^y`'s exact cases
are not `pow`'s); the 640-bit `wide` tier is untouched.

**Phase 5 — `erfq`, `erfcq`, then `tgammaq`, `lgammaq`.**  Each larger than
everything above it combined — `f64`'s `gamma.rs` alone is 11 000 lines — with
no table, no generator and no upstream oracle.  `lgammaq`'s reflection formula
needs `sinpiq`, so Phase 2 is a hard prerequisite.  Treat it as a separate
project, and split `erfq`/`erfcq` off from the gamma pair: one minimax family
per band, no reflection, no poles.

One decision worth making before Phase 2 starts: the π-scaled inverses can
fold `1/π` into the tables (exact dyadic offsets, more generated table files)
or divide by π at the end (no new tables, one more rounding to certify).
Folding is the recommendation — it is what makes the offsets exact.

## How the functions work

`sqrtq`, `rsqrtq` and `cbrtq` seed a fixed-point z<sup>&minus;1/2</sup> / z<sup>&minus;1/3</sup>
from degree-2 Taylor tables (function values, no fitted coefficients) and keep
the midpoint walk only for the rounding-tie window their guard bits cannot
decide; `sqrtq` rides the same seed as `rsqrtq` and corrects s = r·z by the
same power series in r²z &minus; 1.
`expq`, `exp2q` and `exp10q` share one fixed-point engine for 2<sup>x·L</sup>,
with a 256-bit accurate leg behind the Ziv gate of the 128-bit fast one.
`expm1q` reuses that engine above 2<sup>&minus;6</sup>; below, a Taylor
correction rides on top of the exact input significand, so subtracting 1 from
2<sup>f</sup> &isin; [1, 2) never gets to cancel the bits that matter.
`exp2m1q` and `exp10m1q` share that subtracted frame with their own reduction
constant. Below 2<sup>&minus;6</sup>, independently generated Taylor constants
for `(b^x − 1)/x` retain the irrational slope: a 128-bit ratio multiplies the
exact input significand, with a 384-bit ratio on fallback. Subnormals round
directly on their final grid. The exact and midpoint cases, including
`exp2m1q(±114)`, pass through the shared integer rounder. Their MPFR corpora
include every input binade, inverse images of output midpoints, continued
fractions of `ln(b)` on the normal and subnormal midpoint grids, thresholds,
seams and 16 million near-midpoint scan inputs per base. In-source soundness
checks measure worst |error|/gate of 0.359738 (`exp2m1q`) and 0.327608
(`exp10m1q`), giving 2.78× and 3.05× margin respectively.
`logq` reduces in log space instead: an 18-bit estimate of
log<sub>2</sub>(m) picks three 31-bit reciprocals whose product with the
significand is exact, so the logarithms to add back are the only table the sum
needs; `log2q` and `log10q` are the same engine with base-2 and base-10
tables, exact at every power of two and every representable power of ten, and
`log1pq` is the natural one behind an exact 256-bit `1 + x` (its argument is
its own reduction below 2<sup>&minus;18</sup>).
`log2p1q` and `log10p1q` share that front end with their own base's constants.
Their small leg retains the input's floating scale and the factor
log<sub>b</sub>(e) all the way down to subnormals, rounding directly on the
subnormal grid when needed. Their MPFR corpora include exact powers, the
transition points, inverse images of rounding midpoints, and scans near −1
and zero. The worst measured |error|/gate is 0.1870 for `log2p1q` and
0.2565 for `log10p1q`, giving more than the required 2× margin.
`powq` is 2<sup>y·log<sub>2</sub>x</sup> on both engines: the logarithm's
frame times the exact significand of `y` feeds the exponential's, with the
Ziv gate widened by `|y|`; a 384-by-256-bit accurate leg decides what the fast
one refuses, an exact-case detector rounds the dyadic-rational powers
(`x^y = m^N·2^k`, the only midpoints there are) from their integer value, and a
table-free 640-bit tier settles the rest.
`atan2q` reduces on dyadic breakpoints: one float divide picks
`i ≈ round(64·min/max)`, the 6-bit dyadic `i/64` makes both sides of
tan(θ &minus; atan(i/64)) exact 128-bit integers, and one hardware 128-by-64
divide seeds the Newton reciprocal that divides them to 128 or 384 bits before
the atan(i/64) table and the quadrant offset add back &mdash; every variable
shift along the way cut out of 64-bit limbs, since LLVM has no 128-bit funnel
shift.

`atanq` rides `atan2q` — `atan2(x, 1)` is exactly `atan(x)` — but folds the
unit operand through its own reduction: the sector is an integer shift
(`|x| < 1`) or one hardware divide (`|x| ≥ 1`) instead of the float divide,
and below the first breakpoint the reduced tangent *is* the input significand,
so both legs skip the quotient and its Newton reciprocal outright.  The shared
legs, tables, and Ziv gate carry over, and the folded path ships its own
soundness certification.

`asinq` and `acosq` split at `|x| = 2^-3`.  Below it the arc sine is its own
reduced argument: no root is formed at all and the fast leg is a minimax
polynomial, in floating form for `asin` and summed into `π/2 ∓ ·` for `acos`.
Independent Remez fits use eleven terms below 2^-4 and fourteen in the top
binade, with exact rational error bounds after coefficient rounding.
Above it the fast leg reduces on dyadic *sine* breakpoints and never divides:
with `u = min(x, √(1−x²))` and
`v = max(·)`, the breakpoint `sin φ_j = j/128` is read off `u`'s top bits, and
`sin(asin(u) − φ_j) = u·cos φ_j − v·(j/128)` makes one side an exact
small-integer product and the other a 256-bit multiply by the tabulated
`cos φ_j`; a seven-term Taylor series finishes the difference, and
`asin(j/128)` adds back in `atan2q`'s frame.  `1 − x²` is exact in fixed point and its root is
`sqrtq`'s own integer frame plus one Newton step against all 256 bits.  The
accurate leg is the `atan2q` pipeline fed a 384-bit root, with its own
reduction, tables, and certified Ziv gate.

`sinq` and `cosq` reduce by Payne–Hanek on 64-bit limbs of 2/π: the window
starts at the limb the exponent points to, everything above the units bit but
its two low bits is a multiple of 4 and drops, and the product with the
significand leaves the quadrant and a fraction — 192 bits on the fast leg,
448 on the accurate one.  The fraction rounds to `n = round(256·x/π)`, a
quadrant and a breakpoint `j·π/256`, and the residual `|g| ≤ 1/256`
normalizes into a floating fraction at its own exponent, so `θ = g·π/2`
keeps full relative precision however close `x` sits to a multiple of π/2.
A degree-four minimax for sine and six Taylor terms for cosine in `θ²`
(eighteen Taylor coefficients each on the accurate leg) and a 128-entry
`sin`/`cos(j·π/256)` table recombine in `atan2q`'s frames; below 2^-8 the
argument is its own reduced angle, below 2^-57 the results are `x` and 1.

`sincosq` is that same pipeline run once: one Payne–Hanek reduction, one
`sin θ` and one `1 − cos θ`, and only the breakpoint recombination happens
twice — the two results differ solely in which quadrant and which table
product they take.  Each half is rounded on its own; if either lands in the
tie window, one wide reduction feeds both accurate legs.

`tanq` shares that reduction and evaluates `tan θ` with a degree-six minimax
polynomial, generated independently with rminimax and certified after fixed-point
rounding. The accurate leg keeps twenty-five Taylor terms from exact Bernoulli
rationals. It then recombines by the addition formula `tan(j·π/256 + θ) =
(T_j + tan θ)/(1 − T_j·tan θ)` from a table of `tan(j·π/256)`: numerator and
denominator never cancel, and an odd quadrant just swaps them (`−cot`).  The
quotient is `atan2q`'s hardware-seeded Newton reciprocal of the denominator's
top limb plus one Newton step on the quotient itself against the full
denominator (two at 384 bits on the accurate leg), every iterate held below
the ratio so no residual goes negative.

`sinpiq`, `cospiq`, and `tanpiq` use those same kernels after an exact
significand split modulo 2. The residual has at most 113 significant bits;
multiplication by the existing 128/384-bit π/2 constant, with one exponent
bit added, gives π times that residual. Tiny odd results stay in floating
integer form through subnormal rounding. Exact table breakpoints carry a
zero residual; the integer/half-integer values and tangent's exact ±1 cases
bypass the series entirely. No new coefficients or tables are needed.

`examples/gen_f128_trigpi_cases.rs` freezes precision-113 MPFR answers with
ternary-aware subnormalization. Its layers are the exact grid and its
neighbors, inverse images of 114-bit output midpoints, continued fractions
of π and π/2 in the linear band, and a 20-million-input midpoint scan per
function. The unconditional corpus gates need no MPFR; the optional sweeps
cover every exponent field and the reduction seams. `trigpi::ziv_soundness`
checks the three complete fast legs against MPFR at 300 bits: worst
`|err|/gate` is 0.206585 / 0.197872 / 0.105193 (sine / cosine / tangent),
so every gate retains more than 4.8× margin.

## Baselines

Three other binary128 implementations are within reach on a GNU/Linux box, and
they are not interchangeable:

- **[CORE-MATH]** shares metallic's ≤ 0.5 ulp contract, so it is the only fair
  performance baseline — it is the only one doing the same work.  It binds
  thirteen of the twenty-three functions above.
- **nightly Rust** adds no binary128 math of its own.  `f128::sin`,
  `f128::powf` and the rest are `extern "C"` calls into **glibc**'s
  `_Float128` libm (`sinf128`, `powf128`, …), and `std` documents their
  precision as "non-deterministic … varies by platform, Rust version, and can
  even differ within the same execution".  `f128::sqrt` is the one method with
  a contract (IEEE 754 `squareRoot`), and on x86-64 it does not reach glibc at
  all: the static linker binds it to `compiler_builtins`' own `sqrtf128`,
  which is why it is the one `f128::` lane within 2× of metallic instead of
  10× off.
- **libquadmath** is GCC's `__float128` runtime: a mechanical 2018 copy of
  glibc's `ldbl-128` sources (fdlibm and Moshier's Cephes) that GCC documents
  no accuracy for whatsoever.  Where it and glibc agree — bit for bit on every
  function here but `sqrt` and `hypot` — that is shared ancestry, not
  independent confirmation, and the two time within a few percent of each
  other.

`examples/f128_ulp_survey.rs` measures all three against MPFR at 300 bits:

```console
$ cargo +nightly run --release --features "f128 mpfr" --example f128_ulp_survey -- 50000
```

Maximum error in ulps over 50 000 draws per function, taken from the benches'
own bands so the accuracy population is the timed one — except `asinq`/`acosq`,
whose bench band never leaves `|x| < ½`, so the survey draws all of `[−1, 1]`
and crowds the endpoints where the reflection cancels.  Correct rounding is
`≤ 0.5`; anything above it is a wrong last bit.

| function | metallic | glibc 2.35 | libquadmath 12.3 |
|----------|:--------:|:----------:|:----------------:|
| `acosq`   | **0.500** | 1.055 | 1.055 |
| `asinq`   | **0.500** | 0.875 | 0.875 |
| `atanq`   | **0.500** | 1.068 | 1.068 |
| `atan2q`  | **0.500** | 1.541 | 1.541 |
| `cbrtq`   | **0.500** | 0.721 | 0.721 |
| `cosq`    | **0.500** | 1.387 | 1.387 |
| `exp10q`  | **0.500** | 1.712 | n/a |
| `exp2q`   | **0.500** | 0.961 | 0.961 |
| `expm1q`  | **0.500** | 1.356 | 1.356 |
| `expq`    | **0.500** | **0.500** | **0.500** |
| `hypotq`  | **0.500** | 0.533 | 1.082 |
| `log2q`   | **0.500** | 0.623 | 0.623 |
| `log10q`  | **0.500** | 0.613 | 0.613 |
| `log1pq`  | **0.500** | 2.327 | 2.327 |
| `logq`    | **0.500** | 0.501 | 0.501 |
| `powq`    | **0.500** | 0.765 | 0.765 |
| `rsqrtq`  | **0.500** | n/a | n/a |
| `sinq`    | **0.500** | 1.095 | 1.095 |
| `sqrtq`   | **0.500** | **0.500** | 0.750 |
| `tanq`    | **0.500** | 0.804 | 0.804 |

The new `log2p1q` and `log10p1q` also stay at or below 0.5 ulp in a separate
200 000-draw survey of the same `log1pq` band. Neither has a glibc or
libquadmath entry in the survey. Reproduce with:

```sh
CC=clang cargo +nightly run --release --features "f128 mpfr" \
  --example f128_ulp_survey -- 200000 log2p1q log10p1q
```

The share of draws whose last bit comes out wrong spans four orders of
magnitude: 0.006% for glibc's `logq`, ~1% for the exponentials and the sine,
3% for `tanq`, 6% for `powq` and the arc functions, 9% for `cbrtq`, 14% for
`atanq`, 17% for `exp10q`, 19% for `atan2q`, and 25% for libquadmath's `sqrtq`
and `hypotq` — neither of which is correctly rounded, while glibc's `sqrtf128`
is (IEEE mandates it) and its `hypotf128` nearly so.  These are maxima over a
random band, not a search: the adversarial lower bounds in Gladman, Innocente,
Mather and Zimmermann's [*Accuracy of Mathematical Functions*][accuracy] —
which glibc's manual now cites in place of its own deleted ulp table — are
larger still for the same glibc functions (3.51 for `log1p`, 30.3 for `pow`),
and every number above sits under them.

[accuracy]: https://inria.hal.science/hal-03141101
