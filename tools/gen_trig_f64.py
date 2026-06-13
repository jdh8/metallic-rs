#!/usr/bin/env python3
"""Generate constants for the f64 trigonometric functions (sin/cos/tan).

Run:  python3 tools/gen_trig_f64.py

Design:
  rem_pio2 reduces x to q (quadrant, mod 4) and r in [-pi/4, pi/4] as a
  double-double.  Then sin/cos use even/odd kernels in u = r*r:
     sin(r) = r * S(u),   cos(r) = C(u),
  with S approximating sin(r)/r and C approximating cos(r) on u in [0, (pi/4)^2].

Outputs (paste into src/f64/trig.rs):
  - SIN_KERNEL, COS_KERNEL: double-double minimax coeffs (~2^-108)
  - SIN_FAST, COS_FAST: plain-f64 coeffs for the fast path (~2^-60)
  - PIO2_1/2/3: 3-word pi/2 for Cody-Waite (each with low mantissa bits clear)
  - FRAC_2_PI: little-endian u64 words of 2/pi for Payne-Hanek
"""
from mpmath import mp, mpf, sin, cos, pi, chebyfit, taylor
import struct

mp.prec = 2200


def f64(x):
    return struct.unpack('<d', struct.pack('<d', float(x)))[0]


def dd(x):
    hi = f64(x)
    lo = f64(mpf(x) - mpf(hi))
    return hi, lo


def emit_dd_array(name, coeffs):
    print(f"const {name}: [DoubleDouble; {len(coeffs)}] = [")
    for c in coeffs:
        hi, lo = dd(c)
        print(f"    DoubleDouble {{ high: {hi!r}, low: {lo!r} }},")
    print("];\n")


def emit_f64_array(name, coeffs):
    print(f"const {name}: [f64; {len(coeffs)}] = [")
    for c in coeffs:
        print(f"    {f64(c)!r},")
    print("];\n")


# Kernel domain: u = r^2 in [0, (pi/4)^2].
U_MAX = (pi / 4) ** 2

# sin(r)/r = 1 - u/6 + u^2/120 - ... ; even, approximate as a polynomial in u.
# cos(r)   = 1 - u/2 + u^2/24 - ...
# chebyfit over [0, U_MAX]; degrees chosen so the residual is ~2^-110.
sin_over_r = lambda u: sin(mpf(u) ** mpf('0.5')) / (mpf(u) ** mpf('0.5')) if u != 0 else mpf(1)
cos_u = lambda u: cos(mpf(u) ** mpf('0.5'))

sin_coeffs, sin_err = chebyfit(sin_over_r, [0, U_MAX], 13, error=True)
cos_coeffs, cos_err = chebyfit(cos_u, [0, U_MAX], 13, error=True)
# chebyfit returns highest-degree first; reverse to low-degree first.
sin_coeffs = sin_coeffs[::-1]
cos_coeffs = cos_coeffs[::-1]

print(f"// sin(r)/r kernel max error ~ {mp.log(sin_err, 2)} bits")
emit_dd_array("SIN_KERNEL", sin_coeffs)
print(f"// cos(r) kernel max error ~ {mp.log(cos_err, 2)} bits")
emit_dd_array("COS_KERNEL", cos_coeffs)

# Fast-path tail kernels.  Peel the THREE exact leading terms into double-double
#   sin(r)/r = 1 - u/6 + u^2/120 + u^3 * SIN_TAIL(u)
#   cos(r)   = 1 - u/2 + u^2/24  + u^3 * COS_TAIL(u)
# and fit only the tiny u^3 remainder TAIL(u) in plain f64.  The peeled terms
# (1, -1/6 or -1/2, 1/120 or 1/24) carry the value to ~2^-60, while the tail
# value is only ~2^-12 so its f64 evaluation rounding (~2^-53 relative) is
# ~2^-65 absolute on the kernel — negligible.  This is what lets the Ziv gate run
# tight; the full f64 poly would be only ~2^-48.  The effective approximation
# error on the kernel value is U_MAX^3 times the fit residual.
sin_tail_fn = lambda u: (sin_over_r(u) - 1 + mpf(u) / 6 - mpf(u) ** 2 / 120) / mpf(u) ** 3 if u != 0 else -mpf(1) / 5040
cos_tail_fn = lambda u: (cos_u(u) - 1 + mpf(u) / 2 - mpf(u) ** 2 / 24) / mpf(u) ** 3 if u != 0 else -mpf(1) / 720
sin_tail, sin_tail_err = chebyfit(sin_tail_fn, [0, U_MAX], 6, error=True)
cos_tail, cos_tail_err = chebyfit(cos_tail_fn, [0, U_MAX], 6, error=True)
print(f"// SIN_TAIL fit ~ {mp.log(sin_tail_err, 2)} bits; on sin(r)/r ~ {mp.log(U_MAX ** 3 * sin_tail_err, 2)} bits")
emit_f64_array("SIN_TAIL", sin_tail[::-1])
print(f"// COS_TAIL fit ~ {mp.log(cos_tail_err, 2)} bits; on cos(r) ~ {mp.log(U_MAX ** 3 * cos_tail_err, 2)} bits")
emit_f64_array("COS_TAIL", cos_tail[::-1])

# The exact u^2 coefficients peeled into the double-double leads, carried as
# double-doubles so the term is not capped at the f64 rounding of 1/120 (1/24).
print()
for name, value in [("FRAC_1_120", mpf(1) / 120), ("FRAC_1_24", mpf(1) / 24)]:
    hi, lo = dd(value)
    print(f"const {name}: DoubleDouble = DoubleDouble {{ high: {hi!r}, low: {lo!r} }};")
print()

# --- pi/2 in FIVE words; PIO2_1..PIO2_4 each have low 21 mantissa bits cleared
# so q * word is exact for |q| < 2^21 (medium range |x| < 2^20); PIO2_5 carries
# the full remainder.  The 5-word split reaches ~199 bits, well past the ~117-bit
# ceiling of a 3-word split, so the reduced angle stays relative-accurate
# (~2^-100) even within ~2^-60 of a multiple of pi/2 — which keeps the relative
# Ziv gate sound there. ---
def clear_low(x, bits):
    b = struct.unpack('<Q', struct.pack('<d', f64(x)))[0]
    b &= ~((1 << bits) - 1)
    return struct.unpack('<d', struct.pack('<Q', b))[0]

half_pi = pi / 2
words = []
rem = mpf(half_pi)
for _ in range(4):
    w = clear_low(rem, 21)
    words.append(w)
    rem = rem - mpf(w)
words.append(f64(rem))  # PIO2_5: full remainder
for k, w in enumerate(words, 1):
    print(f"const PIO2_{k}: f64 = {w!r};")
err = mp.log(abs(half_pi - sum(mpf(w) for w in words)) / half_pi, 2)
print(f"// PIO2_1..5 represents pi/2 to ~{err} bits relative\n")

# --- 2/pi as little-endian u64 words for Payne-Hanek (enough for f64). ---
# We need bits of 2/pi covering exponent (up to ~1024) + 53 + ~128 guard.
NWORDS = 24  # 24*64 = 1536 bits
two_over_pi = 2 / pi
frac = two_over_pi  # 2/pi in [0, 1)
words = []
acc = mpf(two_over_pi)
# extract integer part 0, then fractional bits in 64-bit chunks
acc = acc - int(acc)
for _ in range(NWORDS):
    acc *= mpf(2) ** 64
    w = int(acc)
    words.append(w)
    acc -= w
print(f"const FRAC_2_PI: [u64; {NWORDS}] = [")
# store little-endian: words[0] is the most significant chunk in this list;
# print in the order the reduction code expects (document in trig.rs).
for w in words:
    print(f"    0x{w:016X},")
print("];")

# --- tan fast-path kernel: tan(r) = r * T(v), v = r^2 in [0, (pi/4)^2]. ---
# Like asin, tan has a pole (at pi/2) so T converges only geometrically; peel the
# 6 exact leading terms into double-double and fit the v^6 remainder in plain f64.
# Reconstruction uses tan(r) for even quadrants and -1/tan(r) for odd ones, so the
# kernel must clear the 2^-59 trig gate after a reciprocal (relative error is
# preserved by 1/t): target ~2^-65, a ~64x margin.
from mpmath import tan as mp_tan

TAN_PEEL = 6

# b_k = coefficient of v^k in tan(r)/r; from tan's odd Taylor coefficients.
tan_taylor = taylor(mp_tan, 0, 2 * TAN_PEEL + 40)
def b(k):
    return tan_taylor[2 * k + 1]

def tan_T(v):
    return mp_tan(v ** mpf('0.5')) / v ** mpf('0.5') if v != 0 else mpf(1)

def tan_tail_fn(v):
    if v == 0:
        return b(TAN_PEEL)
    lead = sum(b(k) * mpf(v) ** k for k in range(TAN_PEEL))
    return (tan_T(v) - lead) / mpf(v) ** TAN_PEEL

print()
emit_dd_array("TAN_LEADS", [b(k) for k in range(TAN_PEEL)])

TAN_TARGET = mpf(2) ** -65
for TAN_TAIL_DEG in range(4, 24):
    ttail, ttail_err = chebyfit(tan_tail_fn, [0, U_MAX], TAN_TAIL_DEG + 1, error=True)
    if U_MAX ** TAN_PEEL * ttail_err < TAN_TARGET:
        break
print(f"// TAN_TAIL deg {TAN_TAIL_DEG} fit ~ {mp.nstr(mp.log(ttail_err, 2), 5)} bits;"
      f" on tan(r)/r ~ {mp.nstr(mp.log(U_MAX ** TAN_PEEL * ttail_err, 2), 5)} bits")
emit_f64_array("TAN_TAIL", ttail[::-1])

# Certify the assembled kernel (double-double leads, f64 tail) vs tan over
# r in (0, pi/4]; the f64 evaluation rounding (~2^-62) is verified by cargo test.
leads_dd = [sum(map(mpf, dd(b(k)))) for k in range(TAN_PEEL)]
tail64 = [f64(c) for c in ttail[::-1]]
worst = mpf(0)
N = 200000
for i in range(1, N + 1):
    r = (pi / 4) * mpf(i) / N
    v = r * r
    t = sum(leads_dd[k] * v ** k for k in range(TAN_PEEL))
    t += v ** TAN_PEEL * sum(mpf(tail64[j]) * v ** j for j in range(len(tail64)))
    rel = abs(r * t / mp_tan(r) - 1)
    if rel > worst:
        worst = rel
print(f"// assembled TAN kernel max relative error ~ {mp.nstr(mp.log(worst, 2), 5)} bits"
      f" (trig gate TRIG_ZIV_EPS = 2^-59)")
