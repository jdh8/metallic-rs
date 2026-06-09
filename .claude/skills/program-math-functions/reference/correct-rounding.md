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
**you may lean on FMA freely**: Rust's `f64::mul_add` is a true FMA on every target,
wrapped as `crate::fma`/`crate::fmaf` (call those, not the clippy-denied builtin;
see [exact-arithmetic.md](exact-arithmetic.md)), so FMA-saturated Ziv levels and
compensated evaluation are fine here — just remember the `crate::fast_mul_add`
helper is *not* an EFT.

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

## Generating worst cases for bivariate (and complex) functions

When a function has **no published worst-case table**, you must find the
hard-to-round (HR) inputs yourself. First check whether you actually need to:
CORE-MATH already publishes HR databases for the binary32 functions it covers —
**including the bivariate `powf`, `hypot`, and `atan2`** — so for those, *use the
published cases* (via the `core-math` crate) rather than regenerating. The gap is
functions outside that set: metallic's two-argument **`log(x, base)`** (`= ln x /
ln base`, neither a C99 function nor in CORE-MATH) is the case in point, as the
real/imag components of a future complex function would be.

The approach metallic-rs's own generators (`gen_f32_log_cases`, `gen_f64_log_cases`)
take — scan inputs, keep those whose MPFR value lands within a normalized threshold
of an `f32`/`f64` midpoint, freeze the survivors with their correct answers — is
exactly the **"naive exhaustive search"** the CORE-MATH paper measures and rejects:
an MPFR evaluation per input costs ~37 s to cover *one* binary32 binade,
extrapolating to ~10⁴ core-years for the full 2⁵⁶ regular `powf` pairs. It is fine as
a **regression-guard corpus** (and the only practical option when the kernel is a
double-double ≈2⁻⁹⁴ — already far past the format, so true HR cases are
astronomically rare and a margin-padded near-midpoint scan suffices), but it does
**not** enumerate the true worst cases. When you need that, use the structured
methods below — the paper demonstrates them on `powf` (and `hypot`/`atan2`); none
targets `log(x, base)`, but Algorithm 1 is general and applies. (Source: *The
CORE-MATH Project*, Sibidanov–Zimmermann–Glondu, ARITH 2022, §II — local copy
`~/doc/core-math-final.pdf`.)

**The key reframing.** "There is at present no *general* clever algorithm for HR
search of bivariate functions" (Stehlé's SLZ extension was never implemented;
Brisebarre–Hanrot [2], integer points near a transcendental curve, is the recent
candidate). The practical wins come from **fixing one argument** to make the problem
univariate, then either a fast per-slice scan or a function-specific shortcut.

1. **Fix one argument, scan the other with finite differences (CORE-MATH
   Algorithm 1, `worst_powf`).** For a fixed `y`, you want every `x` with `xʸ`
   m-HR (≥ m identical bits past the round bit). On each binade take a **degree-2
   Taylor** model of `xʸ` with a rigorous error term, reduce the m-HR test to
   `|frac(α + βi + γi²)| < p(i)` in integers **mod 2⁶⁴**, and evaluate that
   quadratic over consecutive `i` by the **table-of-differences method**: keep a
   running value `α′`, first difference `β′`, second difference `γ′` and step with
   just `α′ += β′; β′ += γ′` (two 64-bit adds, ~1 cycle each) — no multiply, no
   per-point polynomial eval. This is ~18.6 cycles per `x` (≈12 s per exponent),
   **~17× faster than the BaCSeL tool**, and applies to *any* bivariate function
   (and to univariate, since the second argument is then just a constant).

2. **Search the inverse where the function contracts.** When one argument makes the
   map contracting (small `|y|` for `powf`, so many `x` collapse to few outputs),
   enumerating the **inverse** (`z^{1/y}`) is cheaper — fewer `z` cover the range.
   CORE-MATH switches to the inverse for `|y| < 2⁻⁹` and to a trivial scan for
   `|y| > 2¹⁴` (few `x` even land in range), Algorithm 1 elsewhere.

3. **Exploit number-theoretic structure when it exists.** Some bivariate functions
   have HR cases pinned by arithmetic, no transcendental search needed:
   - **`hypot`** (`√(x²+y²)`): HR cases are **"almost-Pythagorean triples"**
     `x² + y² = z² ± 1` with `z` exactly representable on 25 bits — test integer
     triples in range instead of scanning floats.
   - **`atan2`**: for each 25-bit value `z`, you need `y/x ≈ tan z`; take the
     **continued-fraction convergents of `tan z`** and keep the last one that is
     exactly a ratio of two representable values — that ratio is the HR input.

4. **Exact and midpoint cases are separate, and cheap.** For round-to-nearest you
   also need ties. In binary32, a **midpoint** is a result representable on **25
   bits but not 24**; an **exact** case is representable outright. Enumerate these
   directly (Lauter–Lefèvre rounding-boundary test [12]) rather than hoping a scan
   lands on them — a `Sum`-collapsing kernel can round a true midpoint either way,
   so they must be in the corpus explicitly.

**Complex functions** are not in the paper, but a complex `f(z)` is a pair of real
**bivariate** functions of `(re, im)` (e.g. `clog`'s real part is
`½·log(x²+y²)` = `log(hypot)`, its imag part is `atan2(y, x)`). Correct rounding is
per-component, so generate worst cases for each component with the methods above —
and since `hypot`/`atan2` are CORE-MATH functions with published HR tables, complex
log/abs/arg can reuse those directly rather than regenerate anything.

**General-purpose HR tools** (reach for these before writing a bespoke search):
**BaCSeL** (Hanrot–Lefèvre–Stehlé–Zimmermann, <https://gitlab.inria.fr/zimmerma/bacsel>)
for one- and two-variable HR search; the **SLZ algorithm** (Stehlé–Lefèvre–Zimmermann,
IEEE TC 2005, lattice reduction) and **Lefèvre's algorithm**
(<https://www.vinc17.net/research/testlibm/>, with the `testlibm` worst-case data) for
univariate slices.

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
