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

# Fast (plain-f64) kernels: lower degree, ~2^-60.
sin_fast, sin_fast_err = chebyfit(sin_over_r, [0, U_MAX], 6, error=True)
cos_fast, cos_fast_err = chebyfit(cos_u, [0, U_MAX], 7, error=True)
print(f"// SIN_FAST max error ~ {mp.log(sin_fast_err, 2)} bits")
emit_f64_array("SIN_FAST", sin_fast[::-1])
print(f"// COS_FAST max error ~ {mp.log(cos_fast_err, 2)} bits")
emit_f64_array("COS_FAST", cos_fast[::-1])

# --- pi/2 in three words, each with low 21 mantissa bits cleared so that
# q * word is exact for |q| < 2^21 (medium range |x| < 2^20). ---
def clear_low(x, bits):
    b = struct.unpack('<Q', struct.pack('<d', f64(x)))[0]
    b &= ~((1 << bits) - 1)
    return struct.unpack('<d', struct.pack('<Q', b))[0]

half_pi = pi / 2
p1 = clear_low(half_pi, 21)
p2 = clear_low(half_pi - p1, 21)
p3 = f64(half_pi - mpf(p1) - mpf(p2))
print(f"const PIO2_1: f64 = {p1!r};")
print(f"const PIO2_2: f64 = {p2!r};")
print(f"const PIO2_3: f64 = {p3!r};")
err = mp.log(abs(half_pi - mpf(p1) - mpf(p2) - mpf(p3)) / half_pi, 2)
print(f"// PIO2_1+2+3 represents pi/2 to ~{err} bits relative\n")

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
