# Approximation: reduction, transforms, evaluation

The job is to replace a transcendental function with a short polynomial (or
rational) that a CPU can evaluate, while keeping the error below your target. Three
moves do almost all the work: **reduce** the argument to a small interval,
**transform** the function so a low-degree approximant fits, and **evaluate** the
polynomial accurately and quickly.

## Argument reduction

Shrink the domain to an interval where a low-degree polynomial converges fast,
using an identity, then reconstruct. Two flavours:

**Additive** — subtract a multiple of a period/scale. `exp2` uses `x = n + r` with
`r ∈ [-½, ½]`, so `2ˣ = 2ⁿ · 2ʳ` (`src/f64/kernel.rs`):

```rust
let n = x.round_ties_even();
let r = x - n;                 // r ∈ [-½, ½]
// 2^r via crate::poly(r, &[...]); then ×2^n via fast_ldexp
```

`exp`-style reductions that subtract `n·ln2` carry `ln2` as a hi+lo pair so
`n * ln2_hi` is exact and the residual `r = (x - n·ln2_hi) - n·ln2_lo` keeps its
low bits.

Trig uses `kernel::rem_pio2(x)` (`src/f32/kernel.rs`), returning the quadrant `q`
(so the caller picks sin/cos and the sign) and the reduced angle as an `f64`. Below
`π·2²⁷` it does the cheap two-word subtract; above it, a Payne–Hanek reduction with
the 256-bit value of 2/π in a `[u64; 4]` and 128-bit integer arithmetic. Getting
the *reduction* accurate over huge arguments is the hard part of trig — that lives
in `rem_pio2`, not in the sin/cos kernels.

**Multiplicative / exponent extraction** — for `log`, pull the binary exponent out
directly. `log2` (`src/f64/kernel.rs`) subtracts a bias centred on √2⁄2 so the
reduced mantissa lands near 1, minimising `|x - 1|`:

```rust
let i = x.to_bits() as i64;
let exponent = (i - consts::FRAC_1_SQRT_2.to_bits() as i64) >> EXP_SHIFT; // ≈ √2/2 bias
let x = f64::from_bits((i - (exponent << EXP_SHIFT)) as u64);            // mantissa near 1
// log2(x) = exponent + 2·log2(e)·atanh((x-1)/(x+1))
```

**Always carry the reduction residual in a hi+lo pair** when the reduction can
cancel catastrophically (large-argument `exp`/trig). The low word is what keeps the
final result faithfully/correctly rounded.

Subnormal inputs: normalise first so one code path handles all magnitudes — see
`normalize()` and the `Magnitude` enum in each `mod.rs`.

## Symmetry transforms (the core trick)

Approximate a *related* function `g` whose Taylor series is shorter and whose
symmetry is exact. From the article:

| `f` is…            | identity              | approximate            | example kernel        |
|--------------------|-----------------------|------------------------|-----------------------|
| through the origin | `f(x) = x·g(x)`       | `g`                    | `atanh`               |
| even               | `f(x) = g(x²)`        | `g`, in `x²`           | `kernel::cos`         |
| odd                | `f(x) = x·g(x²)`      | `g`, in `x²`           | `kernel::sin`         |

Two payoffs: the degree in the working variable roughly halves (a poly in `x²`),
and the symmetry becomes **structurally exact** — an odd `f` written as `x·g(x²)`
is exactly zero at 0 and exactly antisymmetric, with no even-degree term able to
leak in from coefficient rounding.

`kernel::sin` (odd ⇒ `x·g(x²)`, `src/f32/kernel.rs`):

```rust
pub fn sin(x: f64) -> f32 {
    let y = x * x;                       // work in x²
    let y = y * crate::poly(y, &[
        -1.666_666_666_666_663e-1, 8.333_333_333_321_917e-3,
        -1.984_126_982_945_719_3e-4, 2.755_731_358_196_805e-6,
        -2.505_074_230_488_205e-8, 1.589_594_452_434_234_8e-10,
    ]);
    crate::mul_add(y, x, x) as f32        // x + x·g(x²), low word folded in last
}
```

`kernel::cos` (even ⇒ `g(x²)`, value ≈ 1 near origin):

```rust
pub fn cos(x: f64) -> f32 {
    crate::poly(x * x, &[
        1.0, -4.999_999_999_999_946_7e-1, 4.166_666_666_650_087e-2,
        -1.388_888_887_158_942_7e-3, 2.480_157_897_844_104e-5,
        -2.755_529_138_739_507_4e-7, 2.063_333_980_512_758_6e-9,
    ]) as f32
}
```

(Both compute in `f64` and round once to `f32` at the end — see
[exact-arithmetic.md](exact-arithmetic.md) on free f32→f64 widening.)

## Relative vs absolute error

Minimise **relative** error (`|p(x)/f(x) − 1|`), not absolute (`|p(x) − f(x)|`), as
the default — and *especially* for **odd** functions. Floating-point is dense near
zero: a fixed absolute error is many ulps close to the origin and a fraction of a
ulp far out, so only a relative bound stays accurate everywhere, and the origin —
where the representation is densest — is where it matters most. Split by symmetry:

- **Odd** `f(x) = x·g(x²)` passes through zero, so any absolute error there allows
  *unbounded* relative error — you must weight by relative error. Conveniently the
  relative error of `f` equals that of the approximation to `g`, so generate `g`
  under a relative weight.
- **Even** `f(x) = g(x²)` with `g(0) ≠ 0` (e.g. `cos`, value ≈ 1 near the origin) —
  absolute and relative error nearly coincide around zero, so the choice matters
  less, though relative is still the safe default.

In the tools this is the **weight**: rminimax's default weight is the reciprocal of
the function (= relative error); Sollya/Remez take an explicit weight. See
[coefficients.md](coefficients.md).

## Polynomial vs rational

A rational `P(x)/Q(x)` can hit a given accuracy at lower total degree than a single
polynomial, especially near a singularity or a steep region — `atan`'s kernel uses
`fast_polynomial::rational_array`, and `erf`/`atan`-type functions often want a true
rational. The cost is a division. Reach for rational when a polynomial needs an
uncomfortably high degree; otherwise prefer a polynomial. rminimax generates either
(`--num`/`--den`); see [coefficients.md](coefficients.md).

## Evaluation schemes

metallic-rs evaluates with `crate::poly` (= `fast_polynomial::poly_array`) and
`fast_polynomial::rational_array`. That crate already uses a shallow, Estrin/SIMD-
friendly scheme internally, so:

- You **do not hand-roll Horner**, and you do not normally choose Horner-vs-Estrin —
  the library gives you a short dependency chain for free.
- Coefficients go **low-degree first** in the slice (`c[0]` lowest), matching the
  order `poly_array` expects.
- **Compensated / two-word** evaluation is still your job when the last few terms
  decide the final bit: fold a low word back in with `f64::mul_add` or a `Sum`, as
  the kernels do with the `crate::mul_add(y, x, x)` tail. You rarely need this for
  the whole polynomial — only the leading term(s) and the add-back.

Guidance: get it correct with `crate::poly` first; only restructure (split
even/odd, carry a `Sum`) if a benchmark or a ulp failure says so.

## Reconstruction

Undo the reduction. Exponent injection is the common case — multiply by 2ⁿ by
adding to the bit pattern rather than doing a float multiply:

```rust
kernel::fast_ldexp(mantissa, n)   // adds n << (MANTISSA_DIGITS - 1) to the bit pattern
// or, for a const power:  crate::exp2i(n)
```

Handle the ends explicitly: clamp before reduction to avoid overflow in `n` (see
`exp2`'s `MIN_EXP`/`MAX_EXP` guards), and build subnormal outputs with a dedicated
branch rather than hoping the injection underflows correctly.
