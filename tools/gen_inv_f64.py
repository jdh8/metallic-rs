#!/usr/bin/env python3
"""Generate constants for the f64 inverse trigonometric fast path (asin/acos).

Run:  python3 tools/gen_inv_f64.py

Design (shared by asin's |x|<1/2 branch and both branches of acos, mirroring the
f32 ASIN_NEAR_ZERO kernel):
    asin(t) = t * B(u),   u = t^2 in [0, 1/4],
    B(u) = asin(t)/t = sum_k a_k u^k,   a_k = C(2k,k) / (4^k (2k+1)).
For |x| >= 1/2 the reflection s = sqrt((1-|x|)/2) in [0, 1/2] keeps u in [0, 1/4],
so one kernel serves the whole domain: asin(x) = pi/2 - 2*asin(s).

Because asin's series converges slowly (a_k ~ 1/k), peel the first 5 EXACT terms
into double-double and fit only the u^5 remainder in plain f64:
    B(u) = (1 + a1 u + a2 u^2 + a3 u^3 + a4 u^4) + u^5 * TAIL(u).
The peeled leads carry B to ~2^-105; the tail value is only ~2^-15 so its f64
rounding (~2^-53 relative) is ~2^-68 absolute on B -- a ~30x margin under the
2^-63 Ziv gate (ATAN_ZIV_EPS).

Outputs (paste into src/f64/atan.rs):
  - ASIN_LEADS: double-double [Sum; 5] = exact a_0..a_4
  - ASIN_TAIL:  plain-f64 minimax of the u^5 remainder
"""
from mpmath import mp, mpf, asin, sqrt, binomial, chebyfit
import struct

mp.prec = 2200

PEEL = 6  # number of exact leading terms carried in double-double


def f64(x):
    return struct.unpack('<d', struct.pack('<d', float(x)))[0]


def dd(x):
    hi = f64(x)
    lo = f64(mpf(x) - mpf(hi))
    return hi, lo


def emit_dd_array(name, coeffs):
    print(f"const {name}: [Sum; {len(coeffs)}] = [")
    for c in coeffs:
        hi, lo = dd(c)
        print(f"    Sum {{ high: {hi!r}, low: {lo!r} }},")
    print("];\n")


def emit_f64_array(name, coeffs):
    print(f"const {name}: [f64; {len(coeffs)}] = [")
    for c in coeffs:
        print(f"    {f64(c)!r},")
    print("];\n")


# a_k = C(2k,k) / (4^k (2k+1)); the exact Taylor coefficients of asin(t)/t in u.
def a(k):
    return binomial(2 * k, k) / (mpf(4) ** k * (2 * k + 1))


U_MAX = mpf(1) / 4

# B(u) = asin(sqrt(u))/sqrt(u); the residual after peeling PEEL terms, over u^PEEL.
def B(u):
    return asin(sqrt(u)) / sqrt(u) if u != 0 else mpf(1)


def tail_fn(u):
    if u == 0:
        return a(PEEL)
    lead = sum(a(k) * mpf(u) ** k for k in range(PEEL))
    return (B(u) - lead) / mpf(u) ** PEEL


# Exact leads a_0 .. a_{PEEL-1} as double-doubles.
emit_dd_array("ASIN_LEADS", [a(k) for k in range(PEEL)])

# Fit the tail in plain f64.  Search the smallest degree whose fit residual,
# scaled by U_MAX^PEEL, lands below 2^-70 on B(u) (a comfortable margin under the
# 2^-63 gate, leaving room for the f64 evaluation rounding ~2^-67).
TARGET = mpf(2) ** -70
for TAIL_DEG in range(4, 20):
    tail, tail_err = chebyfit(tail_fn, [0, U_MAX], TAIL_DEG + 1, error=True)
    if U_MAX ** PEEL * tail_err < TARGET:
        break
print(f"// ASIN_TAIL deg {TAIL_DEG} fit ~ {mp.nstr(mp.log(tail_err, 2), 5)} bits;"
      f" on asin(t)/t ~ {mp.nstr(mp.log(U_MAX ** PEEL * tail_err, 2), 5)} bits")
emit_f64_array("ASIN_TAIL", tail[::-1])

# Certify the ASSEMBLED kernel's max relative error vs asin over t in [0, 1/2].
# Leads are double-double (high + low ~ exact to 2^-105); the tail uses f64-rounded
# coefficients.  This captures the approximation error; the separate f64 evaluation
# rounding (tail value ~2^-12, ~2^-67 absolute on B) is verified by `cargo test`.
leads_dd = [sum(map(mpf, dd(a(k)))) for k in range(PEEL)]
tail64 = [f64(c) for c in tail[::-1]]
def kernel(t):
    u = mpf(t) * t
    b = sum(leads_dd[k] * u ** k for k in range(PEEL))
    b += u ** PEEL * sum(mpf(tail64[i]) * u ** i for i in range(len(tail64)))
    return t * b
worst = mpf(0)
N = 200000
for i in range(1, N + 1):
    t = mpf(i) / (2 * N)          # t in (0, 1/2]
    rel = abs(kernel(t) / asin(t) - 1)
    if rel > worst:
        worst = rel
print(f"// assembled kernel max relative error ~ {mp.nstr(mp.log(worst, 2), 5)} bits"
      f" (gate ATAN_ZIV_EPS = 2^-63)")
