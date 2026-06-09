# Exact arithmetic and rounding

The bits you lose to rounding are the bits between a "looks fine" function and a
correctly-rounded one. This file collects the tools for tracking and controlling
that error. In metallic-rs they appear as the `Sum` double-double type, the
`crate::fma`-based error-free transforms, and the hand-compensated tails in the
kernels.

## Rounding modes

IEEE 754's default is **round-to-nearest, ties-to-even** (RN). Its virtue for a
long computation is statistical: rounding errors behave like a random walk, so the
accumulated error of an n-step sum grows like O(√n) instead of the O(n) you get
from a biased mode like round-toward-zero.

**Rust is effectively RN-only.** `std` has no portable `fesetround`; all the
arithmetic operators and the FMA round to nearest. Treat this as a constraint to
exploit, not a limitation: every algorithm here can assume RN, and correct
rounding only has to be proven for one mode (see
[correct-rounding.md](correct-rounding.md)).

## Round-to-odd: the anti-double-rounding mode

Round-to-odd is a non-standard mode that is invaluable as an *intermediate* step.
It rounds an inexact result to whichever of the two neighbours has an **odd**
significand (and leaves exact values alone).

Why it matters: when you compute in a wider precision and then round again to the
target, a naive round-RN-then-round-RN can **double-round** (round twice past a
midpoint and land on the wrong side). Rounding the intermediate to *odd* prevents
this, because only an even-significand value can be the exact midpoint of two
target numbers — round-to-odd never produces such a midpoint, so the final RN step
is unambiguous. This is the trick behind correctly-rounded `fma`/`sqrt` and behind
RLIBM-ALL's "two extra bits, round-to-odd" construction. In Rust you reach for it
when collapsing a `Sum`/double-double back to a single `f64` near a rounding
boundary.

- *When double rounding is odd* — <https://hal.inria.fr/inria-00070603v2/document>
- GCC's use of it — <https://www.exploringbinary.com/gcc-avoids-double-rounding-errors-with-round-to-odd/>

## Error-free transforms

These compute `op(a, b)` **and** the rounding error exactly, as a pair `(s, e)`
with `s + e == a op b` mathematically and `s = fl(a op b)`.

**TwoProduct via FMA — the default here.** Rust's `f64::mul_add` is a true,
correctly-rounded FMA on every target; the crate exposes it as `crate::fma`
(call that, not the clippy-denied builtin), so the exact product is one line:

```rust
let s = a * b;
let e = crate::fma(a, b, -s);   // s + e == a * b exactly
```

This is exactly what `Sum::from_product` does (`src/f64/kernel.rs`):

```rust
pub fn from_product(x: f64, y: f64) -> Self {
    let high = x * y;
    let low = crate::fma(x, y, -high);   // true FMA — the exact tail
    Self { high, low }
}
```

> Use `crate::fma` (`crate::fmaf` for f32, true FMA) in every error-free
> transform — never the builtin `f64::mul_add`/`f32::mul_add`, which clippy denies.
> Do **not** use the `crate::fast_mul_add` helper here: it degrades to `x * y + a`
> without the `fma` target feature and would silently destroy the `e` you are
> trying to capture. `crate::fast_mul_add` is for hot polynomial spots where a lost
> low bit is acceptable, not for EFTs.

**Fast2Sum** (Dekker) — requires `|a| >= |b|` (or `exp(a) >= exp(b)`). This is
`fast_sum` in `src/f64/kernel.rs`:

```rust
pub const fn fast_sum(a: f64, b: f64) -> Sum {
    let high = a + b;
    let low = a - high + b;   // high + low == a + b exactly
    Sum { high, low }
}
```

**2Sum** (Knuth/Møller) — no ordering requirement, branchless. This is
`Sum::from_sum`:

```rust
pub const fn from_sum(x: f64, y: f64) -> Self {
    let high = x + y;
    let diff = high - x;
    let low = y - diff + (diff - high + x);   // high + low == x + y exactly
    Self { high, low }
}
```

Use `from_sum` (2Sum) when you cannot guarantee operand ordering; `fast_sum`
(Fast2Sum) when you can (it is cheaper). Both are proven robust under RN — *On the
robustness of the 2Sum and Fast2Sum algorithms*,
<https://hal-ens-lyon.archives-ouvertes.fr/ensl-01310023v2/document>.

**Dekker split — only without true FMA.** The C/WASM sibling splits each operand
into two ~27-bit halves to multiply without FMA. You almost never need this in
Rust because `crate::fma` is always a real FMA; reach for it only if you are
deliberately writing a path that must run when even software FMA is too slow.

**Free exact f32 products.** A product of two `f32` is exact in `f64`:
`f64::from(a) * f64::from(b)` loses nothing. Widening to `f64` is often the
cheapest "extra precision" for an `f32` kernel — see how the f32 kernels compute
in `f64` and cast back at the end.

## Double-double (hi + lo) values

A value too precise for one `f64` is carried as `Sum { high, low }` with
`|low| <= ½ ulp(high)` when normalized. metallic-rs has a real type for this
(`src/f64/kernel.rs`) with `Mul`/`Div` operators:

- `Sum::from_product`, `Sum::from_quotient` build a normalized pair from a single
  product/quotient.
- `Sum * Sum`, `Sum * f64`, `Sum / f64` propagate the low word. These "break
  normality" deliberately — they are intermediate-format operations; renormalize
  (`from_sum`) only when you need a clean pair, since the cost usually outweighs
  the gain for < 1 ulp work.
- Hand-compensated tails: add the polynomial result to the low word *before* the
  high word, so the largest-magnitude term rounds last (compensated summation by
  hand). The `crate::fast_mul_add(y, x, x)` tails in `atanh`/`sin` fold the low
  word back this way.

## Practical guidance

- Decide where you need extra precision and use a hi+lo pair *only there* (usually:
  the argument-reduction residual, and the final add-back). Carrying `Sum`
  everywhere is slow and rarely necessary for < 1 ulp.
- Keep the largest-magnitude term for last in a hand-compensated sum.
- In EFTs and compensation, always use `crate::fma` / `crate::fmaf` (true FMA),
  never `crate::fast_mul_add`.
- For an `f32` function, computing the kernel in `f64` and rounding once at the end
  is the simplest route to a correctly-rounded `f32`.
- When you genuinely need a correctly-rounded last step from a wider intermediate,
  round-to-odd the intermediate first.
