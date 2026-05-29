# Correct rounding

**The target for this library is correct rounding: error ≤ 0.5 ulp** — the returned
value is *the* nearest representable to the exact result (ties to even),
indistinguishable from computing in infinite precision and rounding once. This is a
step up from *faithful* rounding (< 1 ulp, i.e. one of the two nearest). metallic-rs
takes ≤ 0.5 ulp as its primary goal and treats a merely-faithful function as a bug
to fix; the C sibling [metallic](https://github.com/jdh8/metallic) is moving the
same way and ports its kernels from here. A function not yet correctly rounded
should be labelled WIP, not shipped as "done".

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
   is a `Sum`/double-double evaluation — see [exact-arithmetic.md](exact-arithmetic.md)).
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
many formats. Rust (like WASM) effectively has exactly **one** mode —
round-to-nearest-ties-to-even — with no portable `fesetround`. Targeting only RN is
a real structural advantage:

- only RN midpoints are hard cases (not the directed-mode boundaries as well), so
  there is more approximation slack → potentially lower-degree, faster kernels;
- none of the round-to-odd-in-two-extra-bits machinery that multi-mode libraries
  carry is needed (round-to-odd stays a tool for collapsing a `Sum`, not a
  requirement);
- the proof obligation is one mode, not four.

So a from-scratch, **RN-only, correctly-rounded** kernel can plausibly be *faster
than CORE-MATH* while matching its accuracy — that is the opening this library takes,
rather than copying a general-purpose design wholesale. And unlike the WASM sibling,
**you may lean on FMA freely**: Rust's `f64::mul_add` is a true FMA on every target
(see [exact-arithmetic.md](exact-arithmetic.md)), so FMA-saturated Ziv levels and
compensated evaluation are fine here — just remember the `crate::mul_add` helper is
*not* an EFT.

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
patterns exist. `tests/f32_univariate.rs` already loops **every** `f32`
(`(0..=u32::MAX).map(f32::from_bits)`) and compares the metallic function against the
`core-math` oracle via the `Identity` trait in `tests/common/mod.rs` (`is` =
bit-equal, with NaNs equal). A clean sweep is a *proof* of correct rounding and is
the CI gate; it runs natively in minutes. To add a function, add a
`test_identity(metal::foo, core_math::foof)` test in the same file (and
`tests/f32_bivariate.rs` for two-argument functions like `hypot`).
`common::truncate_errors` caps the report at 250 mismatches so a regression fails
fast.

**`f64` (binary64): cannot brute-force** (2⁶⁴ inputs). You need the worst-case
hardness for the function and interval. **Use the published worst cases**
(Lefèvre–Muller tables, CORE-MATH's databases) rather than rediscovering them; then
prove (Gappa) that the kernel's error stays under the resulting bound away from
those cases, and handle the listed hard cases explicitly. Until that proof exists,
sample heavily — uniform random, near-boundary, and known-tricky inputs — via
`tests/f64_univariate.rs`, comparing against `core-math` (or `rug` for ground
truth), and track the maximum observed ulp error. Curated hard cases live in
`tests/cases/*.wc` (see `tests/cases/README.md`); `common::parse_case_file` loads
them and `common::test_univariate_cases` runs them.

**Watch the feature flag.** Run `cargo test`, **never `cargo test --all-features`**:
the optional `core-math` feature *replaces* metallic's own f32 trig / `powf` with
CORE-MATH implementations, so under `--all-features` the suite would compare
CORE-MATH against itself instead of testing metallic.

## Tooling

- `core-math` crate (CORE-MATH) — the primary oracle and reference.
- `rug` — MPFR-backed arbitrary-precision ground truth for f64.
- Gappa <https://gappa.gitlabpages.inria.fr/> — machine-checked proofs of the
  kernel's rounding-error bound (the ε that Ziv's strategy and the binary64
  argument both depend on).
- Sollya `supnorm` — rigorous bound on the approximation error feeding that proof
  (see [coefficients.md](coefficients.md)).
- `criterion` (`cargo bench`) — confirm a correctly-rounded kernel stays competitive
  with core-math / std / libm.
