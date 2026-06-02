#!/usr/bin/env python3
"""Generate constants for the f64 inverse trigonometric functions.

Run:  python3 tools/gen_atan_f64.py

asin/acos/atan all reduce to one double-double atan kernel:
  atan(x) reduces |x| to [0,1] (atan(x) = pi/2 - atan(1/x) for |x|>1), then a
  cell of width 1/8 centred on c = k/8 (table holds atan(c)); inside the cell
  atan(q) = atan(c) + atan(u), u = (q-c)/(1+q*c), |u| <= 1/16, and the odd
  series atan(u) = u * sum_k (-1)^k u^{2k}/(2k+1) converges in ~14 terms.
  asin(x) = atan(x/sqrt(1-x^2)),  acos(x) = atan(sqrt(1-x^2)/x).

Outputs (paste into src/f64/atan.rs):
  - ATAN_COEFFS: double-double (-1)^k/(2k+1), low-degree first in u^2.
  - ATAN_TABLE: atan(k/8) as double-double for k in 0..=8.
"""
from mpmath import mp, mpf, atan
import struct

mp.prec = 400


def f64(x):
    return struct.unpack('<d', struct.pack('<d', float(x)))[0]


def dd(x):
    hi = f64(x)
    lo = f64(mpf(x) - mpf(hi))
    return hi, lo


print("const ATAN_COEFFS: [Sum; 16] = [")
for k in range(16):
    hi, lo = dd(mpf((-1) ** k) / (2 * k + 1))
    print(f"    Sum {{ high: {hi!r}, low: {lo!r} }},")
print("];\n")

print("const ATAN_TABLE: [Sum; 9] = [")
for k in range(9):
    hi, lo = dd(atan(mpf(k) / 8))
    print(f"    Sum {{ high: {hi!r}, low: {lo!r} }},")
print("];")
