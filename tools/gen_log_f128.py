#!/usr/bin/env python3
"""Generate `src/f128_/log_tables.rs` and `src/f128_/log2_tables.rs`: the
fixed-point constants of the binary128 logarithms.

Everything here is a *mathematical* constant computed from scratch with mpmath:
ln(2), the three levels of 2^(-j/2^k) reciprocal tables, the natural logarithms
of those (rounded) reciprocals, the Taylor coefficients 1/(k+1) of
log(1+z)/z, and a per-bucket linear fit of log2 used only to pick the reduction
index.  No fitted approximation coefficients, no transcription from another
library.

Usage:
    python3 tools/gen_log_f128.py > src/f128_/log_tables.rs
    python3 tools/gen_log_f128.py --base 2 > src/f128_/log2_tables.rs
    python3 tools/gen_log_f128.py --base 10 > src/f128_/log10_tables.rs

`--base b` emits only what changes with the base: the per-exponent constant
(`log_b(2)`), the logarithm tables of the shared reciprocals, and the Taylor
coefficients `log_b(e)/(k+1)`; the reciprocals and the crude fit are the
natural tables' and are reused.
"""
import sys
from mpmath import mp, mpf, log, nint, floor

mp.prec = 2000

LN2 = log(2)

BASE = None
if sys.argv[1:]:
    assert sys.argv[1:2] == ["--base"] and sys.argv[2:] in (["2"], ["10"]), \
        f"usage: {sys.argv[0]} [--base 2|10]"
    BASE = int(sys.argv[2])


def logb(x):
    """The logarithm of `x` in the requested base."""
    return log(x) if BASE is None else log(x, BASE)

# Accurate-leg frame: a value is `S * 2^-FRAME` with `S` a signed 384-bit
# integer.  The fast leg reuses the top 256 bits, i.e. the scale `2^-(FRAME-128)`.
FRAME = 342

# Reciprocal table entries are `round(2^RECIP * 2^(-j/step))`.
RECIP = 31

# The three reduction levels: 6 index bits each, so `j0/64 + j1/4096 +
# j2/262144` covers the leading 18 bits of `log2(m)`.  Level 0 carries a 65th
# entry because the crude estimate reaches `2^18` at the top of `[1, 2)`.
LEVELS = [(64, 65), (4096, 64), (262144, 64)]

# The crude `log2` table is indexed by the 8 bits below the leading one, and
# holds `A << SLOPE_BITS | B` of the linear fit `2^30*log2(m) ~ A + t*B*2^8`.
BUCKETS = 256
SLOPE_BITS = 23
CRUDE_SCALE = 30


def hexs(value, digits):
    """Underscore-grouped hex literal of `value` padded to `digits` nibbles."""
    text = format(value, f"0{digits}x")
    groups = [text[i:i + 16] for i in range(0, len(text), 16)]
    return "0x" + "_".join(groups)


def limbs(value, count):
    """`value` as `count` little-endian 128-bit limbs."""
    assert 0 <= value < 1 << (128 * count), value
    parts = [hexs(value >> (128 * i) & (1 << 128) - 1, 32) for i in range(count)]
    return "[" + ", ".join(parts) + "]"


def reciprocal(step, j):
    """The rounded reciprocal `round(2^RECIP * 2^(-j/step))`."""
    r = int(nint(mpf(2) ** RECIP * mpf(2) ** (-mpf(j) / step)))
    assert 1 << RECIP - 1 <= r <= 1 << RECIP, (step, j, r)
    return r


def recip_table(name, step, count, doc):
    print(f"\n/// {doc}")
    print("#[rustfmt::skip]")
    print(f"pub const {name}: [u32; {count}] = [")
    for row in range(0, count, 8):
        entries = ", ".join(f"0x{reciprocal(step, j):08x}" for j in range(row, min(row + 8, count)))
        print(f"    {entries},")
    print("];")


def log_table(name, step, count, doc):
    """`-log(r/2^RECIP)` for each rounded reciprocal, scaled by 2^FRAME."""
    print(f"\n/// {doc}")
    print("#[rustfmt::skip]")
    print(f"pub const {name}: [[u128; 3]; {count}] = [")
    for j in range(count):
        r = mpf(reciprocal(step, j)) * mpf(2) ** -RECIP
        value = int(nint(-logb(r) * mpf(2) ** FRAME))
        print(f"    {limbs(value, 3)},")
    print("];")


def crude_table():
    """Per-bucket linear fit of `2^30 * log2(m)` over `m` in `[1, 2)`.

    `B` is the secant slope of the bucket and `A` the intercept that centres the
    residual, so the fit is within half the bucket's quadratic sag.  The maximum
    error over all buckets is printed as a doc comment: the reduction index is
    `round(F / 2^12)`, so this bound is in units of `2^-12` of an index step.
    """
    h = mpf(2) ** -8
    scale = mpf(2) ** CRUDE_SCALE
    rows = []
    worst = mpf(0)
    lowest = None
    highest = None

    for i in range(BUCKETS):
        base = 1 + mpf(i) * h
        slope = (log(base + h, 2) - log(base, 2)) / h
        b = int(nint(slope * mpf(2) ** 22))
        assert 0 < b < 1 << SLOPE_BITS, (i, b)

        # `R(t) = 2^30*log2(base + t) - t*b*2^8` is concave: its maximum sits at
        # the tangency point, its minimum at an endpoint.
        def residual(t):
            return scale * log(base + t, 2) - t * mpf(b) * 256

        peak = scale / (mpf(b) * 256 * LN2) - base
        peak = min(max(peak, mpf(0)), h)
        top = residual(peak)
        bottom = min(residual(mpf(0)), residual(h))
        a = int(nint((top + bottom) / 2))
        assert 0 <= a < 1 << (64 - SLOPE_BITS), (i, a)

        worst = max(worst, top - mpf(a), mpf(a) - bottom)
        lowest = mpf(a) - top if lowest is None else min(lowest, mpf(a) - top)
        highest = mpf(a) + mpf(b) if highest is None else max(highest, mpf(a) + mpf(b))
        rows.append(a << SLOPE_BITS | b)

    # The estimate must stay in `[0, 2^18]` after `(F + 2^11) >> 12`.
    assert lowest > -(1 << 11), lowest
    assert (int(highest) + (1 << 11)) >> 12 <= 1 << 18, highest

    print(f"""
/// Linear fit of `2^{CRUDE_SCALE}*log2(m)` per 2^-8-wide bucket of `m` in `[1, 2)`,
/// packed as `intercept << {SLOPE_BITS} | slope`.  The fit is off by at most
/// {float(worst) / 4096:.3f} of a reduction index step, so the reduced `|z|` stays below
/// 2^-18.7 once the index itself is rounded.
#[rustfmt::skip]
pub const CRUDE: [u64; {BUCKETS}] = [""")
    for row in range(0, BUCKETS, 4):
        entries = ", ".join(f"0x{rows[k]:014x}" for k in range(row, row + 4))
        print(f"    {entries},")
    print("];")


if BASE is None:
    print("""// Generated by tools/gen_log_f128.py - do not edit by hand.
//!
//! Fixed-point constants for the binary128 logarithm.  Every entry is a
//! mathematical constant evaluated to 2000 bits with mpmath: ln(2), the three
//! levels of 2^(-j/2^k) reciprocals, the logarithms of those rounded
//! reciprocals, the Taylor coefficients 1/(k+1) of log(1+z)/z, and the linear
//! fit that picks the reduction index.
#![allow(clippy::unreadable_literal)]""".replace("// Generated", "//! Generated"))

    print(f"\n/// ln(2), scaled by 2^{FRAME}.")
    print(f"pub const LN2: [u128; 3] = {limbs(int(nint(LN2 * mpf(2) ** FRAME)), 3)};")

    for level, (step, count) in enumerate(LEVELS):
        recip_table(
            f"RECIP{level}",
            step,
            count,
            f"2^(-j/{step}), scaled by 2^{RECIP} and rounded.",
        )
    name, unit = "log", "1/(k+1)"
else:
    print(f"""// Generated by `tools/gen_log_f128.py --base {BASE}` - do not edit by hand.
//!
//! Fixed-point constants of the binary128 base-{BASE} logarithm that differ from
//! the natural one's: the base-{BASE} logarithms of `log_tables`' rounded
//! reciprocals and the Taylor coefficients log{BASE}(e)/(k+1) of log{BASE}(1+z)/z,
//! every entry evaluated to 2000 bits with mpmath.  The reciprocals and the
//! crude fit are shared with `log_tables`.
#![allow(clippy::unreadable_literal)]""".replace("// Generated", "//! Generated"))

    name = "ONE" if BASE == 2 else f"LOG{BASE}_2"
    exact = f": exactly 2^{FRAME}" if BASE == 2 else ""
    print(f"\n/// log{BASE}(2), scaled by 2^{FRAME}{exact}.")
    print(f"pub const {name}: [u128; 3] = {limbs(int(nint(logb(2) * mpf(2) ** FRAME)), 3)};")
    name, unit = f"log{BASE}", f"log{BASE}(e)/(k+1)"

for level, (step, count) in enumerate(LEVELS):
    log_table(
        f"LOG{level}",
        step,
        count,
        f"-{name}(RECIP{level}[j]/2^{RECIP}), scaled by 2^{FRAME}.",
    )

if BASE is None:
    crude_table()

print(f"""
/// `{unit}` for `k = 0..=13`, scaled by 2^255: the Taylor coefficients of
/// `{name}(1+z)/z = sum_k (-1)^k COEF[k] z^k`.  The fast leg stops at `k = 6`,
/// where the tail is below 2^-152 for `|z| < 2^-18.7`.
#[rustfmt::skip]
pub const COEF: [[u128; 2]; 14] = [""")
for k in range(1, 15):
    print(f"    {limbs(int(nint(logb(mp.e) * mpf(2) ** 255 / k)), 2)},")
print("];")
