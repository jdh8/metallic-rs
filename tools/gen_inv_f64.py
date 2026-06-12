#!/usr/bin/env python3
"""Generate constants for the f64 inverse trigonometric fast path (asin/acos).

Run:  python3 tools/gen_inv_f64.py

Design (shared by asin's |x|<1/2 branch and both branches of acos, mirroring
CORE-MATH's table kernel but evaluated round-to-nearest only):
    asin(t) = t * B(u),   u = t^2 in [0, 1/4],
    B(u) = asin(t)/t = sum_k a_k u^k,   a_k = C(2k,k) / (4^k (2k+1)).
For |x| >= 1/2 the reflection s = sqrt((1-|x|)/2) in [0, 1/2] keeps u in [0, 1/4],
so one kernel serves the whole domain: asin(x) = pi/2 - 2*asin(s), and there
u = (1-|x|)/2 is EXACT (Sterbenz), so no double-double sqrt is needed -- only the
multiplier sqrt(t) = 2s carries a one-division low word.

Tabulate B on a grid u_j = j/128 (j = 0..32) and expand locally:
    u = u_j + r,   r = u - u_j,   |r| <= 1/256,   j = round(128 u),
    B(u) = B(u_j) + r*P_j(r),    P_j = degree-5 per-cell fit.
The leading value B(u_j) is a double-double and P_j has six f64 coefficients,
so one cell is exactly 64 bytes -- one cache line (ASIN_CELLS).

P_j is a CHEBYSHEV interpolant of g(r) = (B(u_j + r) - B(u_j))/r over the
cell (half-cells at j = 0 and j = 32), not a Taylor prefix: near-minimax
fitting pushes the cell-edge truncation (the dominant term of a Taylor P at
r^7) below the f64 evaluation rounding, so the Ziv gate constant scales with
the *rounding* of the correction, not its truncation.  That is what lets the
gate take the form
    eps = |z| * (C*|r| + F) + G
(error sources proportional to the correction d ~ r*P; F covers the
z-proportional floors: the lead's double-double rounding, the sqrt low word,
the product cross terms; G covers the absolute offset-fold rounding in
acos's pi/2 and pi folds).  C, F, G are calibrated below by simulating the
EXACT f64 operation sequence of `asin_tail` in src/f64_/atan.rs over dense
sweeps of all five branch/offset cases, then taking >= 4x margin.

Outputs (paste into src/f64_/atan.rs):
  - ASIN_CELLS: [AsinCell; 33]   = B(u_j) lead + 6 correction coefficients
  - ASIN_EPS_R/Z/ABS             = the calibrated gate constants C, F, G
"""
import math
import random
import struct
from mpmath import mp, mpf, asin, acos, sqrt, binomial, chebyfit

# 260 bits resolves the double-double leads (~2^-106) with wide margin.
mp.prec = 260

N = 128                  # cells per unit of u; u in [0, 1/4] -> j in 0..32
NCELLS = 32 + 1
R_MAX = mpf(1) / (2 * N)  # |r| <= 1/256
DEG = 6                  # correction coefficients per cell (poly degree 5)
KMAX = 220               # series terms; a_k (1/4)^k C(k,m) negligible past this


def f64(x):
    return struct.unpack('<d', struct.pack('<d', float(x)))[0]


def dd(x):
    hi = f64(x)
    lo = f64(mpf(x) - mpf(hi))
    return hi, lo


def td(x):
    hi = f64(x)
    mi = f64(mpf(x) - mpf(hi))
    lo = f64(mpf(x) - mpf(hi) - mpf(mi))
    return hi, mi, lo


# a_k = C(2k,k) / (4^k (2k+1)); the exact Taylor coefficients of asin(t)/t in u.
def a(k):
    return binomial(2 * k, k) / (mpf(4) ** k * (2 * k + 1))


A = [a(k) for k in range(KMAX)]


def B(u):
    u = mpf(u)
    if u == 0:
        return mpf(1)
    t = sqrt(u)
    return asin(t) / t


LEAD = [dd(B(mpf(j) / N)) for j in range(NCELLS)]


def fit_cell(j):
    """Chebyshev interpolant of g(r) = (B(u_j + r) - B(u_j)) / r over the cell.

    Cells 0 and 32 are half-cells (u >= 0 and u <= 1/4 clip them), so the fit
    interval shrinks accordingly and the edge cells come out about 2^6 more
    accurate.  g's removable singularity at r = 0 (an interval endpoint of the
    half-cells) takes its analytic limit B'(u_j) from the series.
    """
    uj = mpf(j) / N
    b0 = B(uj)
    b1 = sum(A[k] * k * uj ** (k - 1) for k in range(1, KMAX))
    lo = mpf(0) if j == 0 else -R_MAX
    hi = mpf(0) if j == 32 else R_MAX

    def g(r):
        return (B(uj + r) - b0) / r if r != 0 else b1

    coeffs = chebyfit(g, [lo, hi], DEG)        # descending order
    poly = [f64(c) for c in reversed(coeffs)]  # ascending: b1 .. b6

    # Certified sup-norm of the f64-rounded fit, in B units per unit r.
    err = mpf(0)
    for i in range(401):
        r = lo + (hi - lo) * mpf(i) / 400
        if r == 0:
            continue
        p = sum(mpf(c) * r ** m for m, c in enumerate(poly))
        err = max(err, abs(p - g(r)))
    return poly, err


CELLS = []
fit_err = mpf(0)
for j in range(NCELLS):
    poly, err = fit_cell(j)
    CELLS.append((LEAD[j], poly))
    fit_err = max(fit_err, err)


# --- exact f64 round-to-nearest simulation (native floats + exact fma) ---
def fma(x, y, z):
    """Correctly-rounded x*y + z using Dekker's exact product and fsum.

    Valid while x*y stays in the normal range and its low word above the
    subnormals (true for every product in the asin tail); self-checked against
    mpmath below.
    """
    if x == 0.0 or y == 0.0:
        return z + x * y  # keep signed-zero semantics
    s = 134217729.0  # 2^27 + 1, Veltkamp split
    xh = x * s - (x * s - x)
    xl = x - xh
    yh = y * s - (y * s - y)
    yl = y - yh
    hi = x * y
    lo = xl * yl - (((hi - xh * yh) - xl * yh) - xh * yl)
    return math.fsum([z, hi, lo])


for _ in range(1000):
    x = random.uniform(-2.0, 2.0)
    y = random.uniform(-2.0, 2.0)
    z = random.uniform(-2.0, 2.0) * 2.0 ** random.randint(-60, 0)
    assert fma(x, y, z) == f64(mpf(x) * mpf(y) + mpf(z)), (x, y, z)


def poly5(x, c):
    """Mirror fast_polynomial::polynomials::poly_5 (crate::poly on 6 coeffs)."""
    x2 = x * x
    x4 = x2 * x2
    return fma(x4, fma(x, c[5], c[4]),
               fma(x2, fma(x, c[3], c[2]), fma(x, c[1], c[0])))


def asin_tail(off, z, zl, r, j):
    """Mirror `asin_tail` in src/f64_/atan.rs: v = off + B(u_j + r)*(z + zl)."""
    (lh, ll), poly = CELLS[j]
    ph = lh * z
    pl = fma(lh, z, -ph)
    low = fma(lh, zl, pl)
    low = fma(ll, z, low)
    low = fma(poly5(r, poly), r * z, low)
    sh = off[0] + ph                  # fast_sum: |off| >= |p| or off == 0
    sl = (off[0] - sh) + ph
    vlow = sl + ((off[1] + low))
    return sh, vlow


def reduce_direct(x):
    jf = f64(128.0 * (x * x))
    jf = float(mp.nint(mpf(jf)))      # round_ties_even on an exact small value
    r = fma(x, x, -(jf * 0.0078125))
    return r, int(jf)


def reduce_reflect(a):
    t = 2.0 * (1.0 - a)               # exact (Sterbenz)
    zm = math.sqrt(t)                 # one RN sqrt
    zl = fma(zm, zm, -t) * ((-0.5 / t) * zm)
    u = 0.25 * t                      # exact
    jf = float(mp.nint(mpf(f64(128.0 * u))))
    r = u - jf * 0.0078125            # exact
    return zm, zl, r, int(jf)


PI2 = dd(mp.pi / 2)
PI_DD = dd(mp.pi)
ZERO = (0.0, 0.0)
TINY = float.fromhex('0x1.7137449123ef6p-26')

# (label, draw x, evaluate one point -> (err, |z|, |r|))
mp.prec = 160  # reference precision for the error measurements


def measure(off, z, zl, r, j, target):
    vh, vl = asin_tail(off, z, zl, r, j)
    err = abs(mpf(vh) + mpf(vl) - target)
    return err, abs(z), abs(r)


def case_asin_direct(x):
    r, j = reduce_direct(x)
    return measure(ZERO, x, 0.0, r, j, asin(mpf(x)))


def case_asin_reflect(x):
    zm, zl, r, j = reduce_reflect(abs(x))
    sgn = -1.0 if x > 0 else 1.0      # z = copysign(sqrt(t), -x)
    off = PI2 if x > 0 else (-PI2[0], -PI2[1])
    return measure(off, sgn * zm, sgn * zl, r, j, asin(mpf(x)))


def case_acos_direct(x):
    r, j = reduce_direct(x)
    return measure(PI2, -x, -0.0, r, j, acos(mpf(x)))


def case_acos_reflect(x):
    zm, zl, r, j = reduce_reflect(abs(x))
    sgn = 1.0 if x > 0 else -1.0      # z = copysign(sqrt(t), x)
    off = ZERO if x > 0 else PI_DD
    return measure(off, sgn * zm, sgn * zl, r, j, acos(mpf(x)))


def draw_direct(n):
    for _ in range(n):
        yield random.uniform(-0.5, 0.5)
    for _ in range(n // 4):           # log-uniform magnitudes down to TINY
        e = random.uniform(math.log2(TINY), -1.001)
        yield math.copysign(2.0 ** e * random.uniform(1.0, 2.0) / 2.0,
                            random.choice((-1.0, 1.0)))


def draw_reflect(n):
    for _ in range(n):
        yield random.uniform(0.5, 1.0) * random.choice((-1.0, 1.0))
    for _ in range(n // 4):           # cluster at both ends of the band
        d = 2.0 ** random.uniform(-50, -1)
        yield random.choice((0.5 + d, 1.0 - d)) * random.choice((-1.0, 1.0))


random.seed(0x5eed)
SAMPLES = []
for x in draw_direct(60000):
    if abs(x) >= TINY:
        SAMPLES.append(case_asin_direct(x))
        SAMPLES.append(case_acos_direct(x))
for x in draw_reflect(60000):
    if abs(x) < 1.0:
        SAMPLES.append(case_asin_reflect(x))
        SAMPLES.append(case_acos_reflect(x))

# Calibrate eps = |z|*(C*|r| + F) + G: C from the r-proportional region, F from
# the tiny-r region, G from the tiny-z region, each with >= 4x margin.
G_FLOOR = mpf(2) ** -100
F_FLOOR = mpf(2) ** -78
c_req = max((err / (z * r) for err, z, r in SAMPLES
             if z > 0 and r >= 2.0 ** -25), default=mpf(0))
f_req = max((err / mpf(z) for err, z, r in SAMPLES if z > 0 and r < 2.0 ** -25),
            default=mpf(0))
g_req = max((err for err, z, r in SAMPLES if z < 2.0 ** -25), default=mpf(0))

C = f64(mpf(2) ** mp.ceil(mp.log(4 * c_req, 2)))
F = f64(max(mpf(2) ** mp.ceil(mp.log(4 * f_req, 2)) if f_req > 0 else F_FLOOR,
            F_FLOOR))
G = f64(max(mpf(2) ** mp.ceil(mp.log(4 * g_req, 2)) if g_req > 0 else G_FLOOR,
            G_FLOOR))

worst = max(err / (mpf(z) * (mpf(C) * mpf(r) + mpf(F)) + mpf(G))
            for err, z, r in SAMPLES)

print(f"// per-cell Chebyshev fit (degree {DEG - 1}): sup error"
      f" ~2^{mp.nstr(mp.log(fit_err, 2), 4)} per unit r")
print(f"// calibration over {len(SAMPLES)} points, all five branch/offset"
      f" cases:")
print(f"//   C >= {mp.nstr(c_req, 4)} (~2^{mp.nstr(mp.log(c_req, 2), 4)}),"
      f" F >= {mp.nstr(f_req, 4)}, G >= {mp.nstr(g_req, 4)}")
print(f"//   worst err/eps with the emitted constants:"
      f" {mp.nstr(worst, 4)} (margin {mp.nstr(1 / worst, 4)}x)")
print(f"const ASIN_EPS_R: f64 = {C!r}; // C")
print(f"const ASIN_EPS_Z: f64 = {F!r}; // F")
print(f"const ASIN_EPS_ABS: f64 = {G!r}; // G\n")

print(f"const ASIN_CELLS: [AsinCell; {NCELLS}] = [")
for (hi, lo), poly in CELLS:
    row = ", ".join(repr(c) for c in poly)
    print("    AsinCell {")
    print(f"        lead: DoubleDouble {{ high: {hi!r}, low: {lo!r} }},")
    print(f"        poly: [{row}],")
    print("    },")
print("];\n")

mp.prec = 260

# --- Triple-double accurate series for the |x| < 1/2 fallback (asin only) ---
# A handful of f64 hard-to-round acos points need the result to ~2^-111, beyond
# what the double-double accurate path resolves.  For the direct branch the
# fallback evaluates asin(t) = t*B(t^2), B(u) = sum_k a_k u^k, in TRIPLE-double:
#   B = 1 + u*(a1 + u*(a2 + u*(... )))   Horner,
# with a1, a2, a3 carried to a third limb (their double-double rounding alone
# would cap B near 2^-110) and the slowly-decaying tail a4.. in double-double.
# NLEAD leading coefficients are carried as triple-doubles (their dd rounding
# alone would cap B above 2^-120 at u=1/4); the rest run double-double Horner.
NLEAD = 8
# K is chosen so the truncated tail a_K (1/4)^K lands below 2^-130 at u = 1/4.
UMAX = mpf(1) / 4
KACC = NLEAD
while A[KACC] * UMAX ** KACC > mpf(2) ** -130:
    KACC += 1
# Floor from the first double-double (non-triple) coefficient a_{NLEAD+1}: its
# ~2^-106 rounding scaled by its term value at u=1/4.
floor = A[NLEAD + 1] * UMAX ** (NLEAD + 1) * mpf(2) ** -106
print(f"// triple-double accurate series: {KACC} terms, {NLEAD} triple-double leads;"
      f" tail ~ 2^{mp.nstr(mp.log(A[KACC] * UMAX ** KACC, 2), 4)},"
      f" dd-lead floor ~ 2^{mp.nstr(mp.log(floor, 2), 4)} at u=1/4")

# a_1..a_KACC as double-double, low-degree first.
print(f"const ASIN_ACC: [DoubleDouble; {KACC}] = [")
for k in range(1, KACC + 1):
    hi, lo = dd(a(k))
    print(f"    DoubleDouble {{ high: {hi!r}, low: {lo!r} }},")
print("];\n")

# Third limbs of a_1..a_NLEAD (carried to triple-double):
# a_k = ASIN_ACC[k-1].high + .low + ASIN_ACC_LO[k-1].
print(f"const ASIN_ACC_LO: [f64; {NLEAD}] = [")
for k in range(1, NLEAD + 1):
    print(f"    {td(a(k))[2]!r},")
print("];")
