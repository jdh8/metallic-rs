#!/usr/bin/env python3
"""Generate binary32 erf/erfc's original, direct erfc Chebyshev table.

Requires mpmath. Run from any directory; the marked table in src/f32_/erf.rs
is replaced. Each degree-10 polynomial approximates erfc((n + h)/16) on
[-1/2, 1/2], for n = 7..161. Coefficients are independently fitted at 100
decimal digits and rounded to binary64. No other library supplies coefficients.
"""

from pathlib import Path

import mpmath as mp


mp.mp.dps = 100
START = "// BEGIN GENERATED ERFC_F32_TABLE"
END = "// END GENERATED ERFC_F32_TABLE"


def main():
    rows = []
    worst = mp.mpf(0)
    analytic = mp.mpf(0)
    arithmetic = mp.mpf(0)
    unit_roundoff = mp.mpf(2) ** -53
    gamma64 = 64 * unit_roundoff / (1 - 64 * unit_roundoff)
    for n in range(7, 162):
        target = lambda h: mp.erfc((n + h) / 16)
        original = list(reversed(mp.chebyfit(target, [-mp.mpf('0.5'), mp.mpf('0.5')], 11)))
        coefficients = [float(c) for c in original]

        # Degree-10 Chebyshev interpolation: the nodal product on [-1/2,1/2]
        # is bounded by 2^-21. The 11th derivative is
        # -2/sqrt(pi) * 16^-11 * H_10(x) * exp(-x^2). Bound exp at the lower
        # cell edge and |H_10| by its absolute coefficients at the upper edge.
        lo, hi = (mp.mpf(n) - mp.mpf('0.5')) / 16, (mp.mpf(n) + mp.mpf('0.5')) / 16
        hermite = sum(mp.factorial(10) / mp.factorial(k) / mp.factorial(10 - 2 * k)
                      * (2 * hi) ** (10 - 2 * k) for k in range(6))
        derivative = 2 / mp.sqrt(mp.pi) / 16**11 * mp.exp(-lo * lo) * hermite
        interpolation = derivative * mp.mpf(2)**-21 / mp.factorial(11)

        # Charge actual coefficient rounding and a deliberately loose gamma64
        # bound on evaluating the degree-10 polynomial. Every term's arithmetic
        # path in poly_array (including power construction) has fewer than 64
        # rounded operations, with or without FMA; all intermediates are normal.
        coefficient_error = sum(abs(mp.mpf(c) - exact) * mp.mpf('0.5')**i
                                for i, (c, exact) in enumerate(zip(coefficients, original)))
        magnitude = sum(abs(mp.mpf(c)) * mp.mpf('0.5')**i for i, c in enumerate(coefficients))
        rounding = coefficient_error + gamma64 * magnitude
        minimum = mp.erfc(hi)
        analytic = max(analytic, (interpolation + rounding) / minimum)
        arithmetic = max(arithmetic, rounding / minimum)
        # Check the rounded coefficients, including both cell endpoints.
        for j in range(129):
            h = mp.mpf(j) / 128 - mp.mpf('0.5')
            value = mp.polyval(list(reversed(coefficients)), h)
            worst = max(worst, abs(value / target(h) - 1))
        rows.append("    [\n" + "".join(f"        {repr(c)},\n" for c in coefficients) + "    ],")

    # The runtime gate scales by the computed result. If the relative error to
    # truth is bounded by b, relative error to the computed value is b/(1-b).
    assert analytic / (1 - analytic) < mp.mpf(2)**-39
    generated = "\n".join([
        START,
        "/// Direct `erfc((n+h)/16)` Chebyshev fits, `n = 7..161`, `|h| ≤ 1/2`.",
        "///",
        "/// Generated independently with `python3 tools/gen_erf_f32.py`:",
        "/// `mpmath.chebyfit(lambda h: erfc((n+h)/16), [-0.5, 0.5], 11)`",
        "/// at 100 decimal digits, then rounded to f64. Do not edit by hand.",
        f"/// Rounded-coefficient sampled relative error: `2^{float(mp.log(worst, 2)):.3f}`.",
        f"/// Analytic interpolation + coefficient/evaluation bound: `2^{float(mp.log(analytic, 2)):.3f}`",
        f"/// relative; the coefficient/evaluation allowance alone is `2^{float(mp.log(arithmetic, 2)):.3f}`.",
        "/// The generator bounds the 11th derivative with the absolute Hermite",
        "/// coefficients, then adds coefficient rounding and a conservative gamma64.",
        "const ERFC_F32_TABLE: [[f64; 11]; 155] = [",
        *rows,
        "];",
        END,
    ])
    path = Path(__file__).resolve().parents[1] / "src/f32_/erf.rs"
    source = path.read_text()
    if START in source:
        before, rest = source.split(START, 1)
        _, after = rest.split(END, 1)
        source = before + generated + after
    else:
        source += "\n" + generated + "\n"
    path.write_text(source)
    print(f"155 degree-10 cells; rounded-coefficient max relative error 2^{float(mp.log(worst, 2)):.3f}")
    print(f"Analytic relative bound 2^{float(mp.log(analytic, 2)):.3f}; ratio to 2^-38 gate {float(analytic * 2**38):.6f}")


if __name__ == "__main__":
    main()
