#!/usr/bin/env python3
"""Generate constants for the f64 inverse trigonometric fast path (asin/acos).

Run:  python3 tools/gen_inv_f64.py

Design (shared by asin's |x|<1/2 branch and both branches of acos, mirroring
CORE-MATH's acos table kernel but evaluated round-to-nearest only):
    asin(t) = t * B(u),   u = t^2 in [0, 1/4],
    B(u) = asin(t)/t = sum_k a_k u^k,   a_k = C(2k,k) / (4^k (2k+1)).
For |x| >= 1/2 the reflection s = sqrt((1-|x|)/2) in [0, 1/2] keeps u in [0, 1/4],
so one kernel serves the whole domain: asin(x) = pi/2 - 2*asin(s), and there
u = (1-|x|)/2 is EXACT (Sterbenz), so no double-double sqrt is needed -- only the
multiplier sqrt(t) = 2s carries a one-division low word.

Rather than one slowly-converging series over the whole [0, 1/4] (the old 5
double-double leads + degree-13 tail), tabulate B on a grid u_j = j/128
(j = 0..32) and expand locally:
    u = u_j + r,   r = u - u_j,   |r| <= 1/256,   j = round(128 u),
    B(u) = B(u_j) + r*(b1 + b2 r + ... + bD r^(D-1)),
    b_m = B^(m)(u_j)/m! = sum_{k>=m} a_k C(k,m) u_j^(k-m).
The leading value B(u_j) is carried as a double-double (ASIN_LEAD); the tiny
correction d = r*P(r) is plain f64 (ASIN_POLY).  Because |r| <= 2^-8 and the
local derivatives are O(1), a low-degree P reaches far below the Ziv gate while
the correction value stays ~2^-10, so its f64 evaluation rounding is ~2^-63.

Outputs (paste into src/f64/atan.rs):
  - ASIN_LEAD: [DoubleDouble; 33] = B(u_j) to ~2^-106
  - ASIN_POLY: [[f64; DEG]; 33]   = per-cell correction coefficients b_1..b_DEG
"""
from mpmath import mp, mpf, asin, sqrt, binomial

# 260 bits resolves the double-double leads (~2^-106) and a ~2^-64 certified
# error with wide margin, while keeping the dense asin sweep fast.
mp.prec = 260

N = 128                  # cells per unit of u; u in [0, 1/4] -> j in 0..32
NCELLS = 32 + 1
R_MAX = mpf(1) / (2 * N)  # |r| <= 1/256
KMAX = 220               # series terms; a_k (1/4)^k C(k,m) negligible past this


def f64(x):
    import struct
    return struct.unpack('<d', struct.pack('<d', float(x)))[0]


def dd(x):
    hi = f64(x)
    lo = f64(mpf(x) - mpf(hi))
    return hi, lo


# a_k = C(2k,k) / (4^k (2k+1)); the exact Taylor coefficients of asin(t)/t in u.
def a(k):
    return binomial(2 * k, k) / (mpf(4) ** k * (2 * k + 1))


A = [a(k) for k in range(KMAX)]


# b_m = B^(m)(u_j)/m! = sum_{k>=m} a_k C(k,m) u_j^(k-m): the m-th local Taylor
# coefficient of B around the cell center u_j = j/128.
def local_coeff(j, m):
    uj = mpf(j) / N
    return sum(A[k] * binomial(k, m) * uj ** (k - m) for k in range(m, KMAX))


def emit_lead(name):
    print(f"const {name}: [DoubleDouble; {NCELLS}] = [")
    for j in range(NCELLS):
        hi, lo = dd(local_coeff(j, 0))
        print(f"    DoubleDouble {{ high: {hi!r}, low: {lo!r} }},")
    print("];\n")


def emit_poly(name, deg):
    print(f"const {name}: [[f64; {deg}]; {NCELLS}] = [")
    for j in range(NCELLS):
        row = ", ".join(f"{f64(local_coeff(j, m))!r}" for m in range(1, deg + 1))
        print(f"    [{row}],")
    print("];\n")


# --- f64 operation simulation (round-to-nearest) for end-to-end certification ---
def rn(x):
    return mpf(f64(x))


def fma(a, b, c):           # correctly-rounded fused multiply-add
    return rn(mpf(a) * mpf(b) + mpf(c))


def poly_f64(x, coeffs):    # Horner in f64 via FMA (proxy for crate::poly's Estrin)
    acc = mpf(coeffs[-1])
    for c in reversed(coeffs[:-1]):
        acc = fma(acc, x, c)
    return acc


# The f64-rounded table entries, degree-independent for the leads.
LEAD = [dd(local_coeff(j, 0)) for j in range(NCELLS)]           # (hi, lo) per cell


def cell(u_f64):            # (j, r): u rounded to a cell and its exact-in-f64 residual
    j = max(0, min(32, int(mp.nint(N * rn(u_f64)))))
    return j, mpf(j) / N


# Compute B(u_j + r) as the f64 pair (hi, lo): hi = LEAD.hi, lo = LEAD.lo + d,
# d = r * P(r).  Mirrors `asin_b` in src/f64/atan.rs.
def asin_b(r, j, poly):
    d = rn(mpf(r) * poly_f64(r, poly[j]))
    lo = rn(mpf(LEAD[j][1]) + d)
    return mpf(LEAD[j][0]), lo                                  # (b_hi, b_lo) exact mpf


# End-to-end max relative error of the f64 fast leg vs asin, over BOTH branches:
# the direct |x|<1/2 path and the reflection [1/2, 1) path (which carries the 2x
# cancellation of pi/2 - sqrt(t)*B near 1/2).  This is the number the Ziv gate
# (ASIN_ZIV_EPS) must dominate.
PI2_HI, PI2_LO = dd(mp.pi / 2)


def certify_f64(deg):
    poly = [[f64(local_coeff(j, m)) for m in range(1, deg + 1)]
            for j in range(NCELLS)]
    worst = mpf(0)
    M = 120000
    for i in range(1, M):
        a = rn(mpf(i) / (2 * M))                                # a in (0, 1/2)
        # Direct: asin(a) = a * B(a^2)  via Mul<f64>.
        j, c = cell(rn(a * a))
        r = fma(a, a, -c)
        bhi, blo = asin_b(r, j, poly)
        ph = rn(bhi * a)
        val = mpf(ph) + fma(blo, a, fma(bhi, a, -ph))
        worst = max(worst, abs(val / asin(a) - 1))

        # Reflection: a' in [1/2, 1), asin(a') = pi/2 - sqrt(t)*B(u), t = 2-2a'.
        # The cancellation pi/2 - sqrt(t)*B peaks at a' = 1/2+ (2x amplification),
        # so the sweep must reach down to 1/2, not just near 1.
        ar = rn(mpf('0.5') + mpf(i) / (2 * M))                  # a' in (1/2, 1)
        t = rn(2 - 2 * ar)                                      # exact
        if t == 0:
            continue
        z = rn(mp.sqrt(t))
        zl = rn(fma(z, z, -t) * rn(rn(mpf('-0.5') / t) * z))
        j, c = cell(rn(mpf('0.25') * t))
        r = rn(mpf('0.25') * t - c)
        bhi, blo = asin_b(r, j, poly)
        # z*b via double-double Mul: (bhi,blo) * (z,zl)
        ph = rn(bhi * z)
        lo = fma(blo, z, fma(bhi, zl, fma(bhi, z, -ph)))
        # asin = pi/2 - z*b  (double-double add, dominant terms)
        val = (mpf(PI2_HI) + mpf(PI2_LO)) - (mpf(ph) + lo)
        worst = max(worst, abs(val / asin(ar) - 1))
    return worst


# Find the smallest per-cell degree whose certified approximation error clears a
# comfortable margin under the intended Ziv gate: aim < 2^-64.
TARGET = mpf(2) ** -64
for DEG in range(3, 12):
    poly = [[mpf(f64(local_coeff(j, m))) for m in range(1, DEG + 1)]
            for j in range(NCELLS)]
    err = mpf(0)
    for i in range(1, 60001):
        t = mpf(i) / 120000
        j = max(0, min(32, int(mp.nint(N * t * t))))
        r = t * t - mpf(j) / N
        ld = mpf(LEAD[j][0]) + mpf(LEAD[j][1])
        d = sum(poly[j][m] * r ** (m + 1) for m in range(DEG))
        err = max(err, abs(t * (ld + d) / asin(t) - 1))
    if err < TARGET:
        break

# End-to-end f64 error (includes evaluation rounding + reflection cancellation).
e2e = certify_f64(DEG)
GATE = mpf(2) ** -59
print(f"// per-cell degree {DEG}: approx error ~ {mp.nstr(mp.log(err, 2), 5)} bits,"
      f" end-to-end f64 ~ {mp.nstr(mp.log(e2e, 2), 5)} bits")
print(f"// -> ASIN_ZIV_EPS = 2^-59 = {f64(GATE)!r} leaves "
      f"~{mp.nstr(mp.log(GATE / e2e, 2), 4)} bits margin")
emit_lead("ASIN_LEAD")
emit_poly("ASIN_POLY", DEG)
