#!/usr/bin/env python3
"""Generate the shared 16-cell table for f32 asinpif/acospif.

Run:  python3 tools/gen_asinpi_f32.py [path-to-ratapprox]

Structure (CORE-MATH's one-step asinpif/acospif shape, our own fits):

- Cell 0 (`|x| < 1/16`): coefficients of `q` in `asinpi(z) ≈ z·q(z²)`,
  fitted on `z² ∈ [0, 2⁻⁸]`.
- Cells 1..15 (`|x| ∈ [i/16, (i+1)/16)`): coefficients of
  `g(a) = (½ − asinpi(a))/√(1−a) = acospi(a)/√(1−a)`, fitted in `a`.

Then `asinpi(x) = sign(x)·(½ − g(|x|)·√(1−|x|))` and
`acospi(x) = g(|x|)·√(1−|x|)` for `x ≥ 0`, `1 − g(|x|)·√(1−|x|)` below.

Each cell is 8 f64 coefficients = one 64-byte cache line.  Fits come from
rminimax's `ratapprox` (relative-error weight, f64 coefficients); this
script shells out to it, then prints the Rust `ASINPI_CELLS` constant.
"""

import subprocess
import sys
import re
import os

RATAPPROX = sys.argv[1] if len(sys.argv) > 1 else os.path.expanduser(
    "~/src/rminimax/build/ratapprox"
)
BASIS = ",".join(["1"] + [f"x^{i}" for i in range(1, 8)])
# 1/16 in these domain strings is exact in decimal.  The last cell must stop
# short of 1, where the g expression is 0/0 (the fit still covers every f32
# input: the largest below 1 is 1 − 2⁻²⁴ < 1 − 10⁻¹², and g is analytic at 1).
CELLS = [("asin(sqrt(x))/(pi*sqrt(x))", "[1e-30,0.00390625]")] + [
    (
        "(0.5 - asin(x)/pi)/sqrt(1-x)",
        f"[{i / 16!r},{'0.999999999999' if i == 15 else repr((i + 1) / 16)}]",
    )
    for i in range(1, 16)
]


def fit(function: str, domain: str) -> list[float]:
    subprocess.run(
        [
            RATAPPROX,
            f"--function={function}",
            f"--dom={domain}",
            f"--num=[{BASIS}]",
            "--den=[1]",
            "--dispCoeff=hex",
            "--log",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    with open("coeffs.sollya", encoding="ascii") as file:
        text = file.read()
    numerator = re.search(r"Numerator = \[\|([^|]*)\|\]", text)
    denominator = re.search(r"Denominator = \[\|([^|]*)\|\]", text)
    assert numerator and denominator
    # ratapprox may emit a scaled constant denominator; divide it out.
    scale = [float.fromhex(h) for h in denominator.group(1).replace(",", " ").split()]
    assert len(scale) == 1
    return [
        float.fromhex(h) / scale[0]
        for h in numerator.group(1).replace(",", " ").split()
    ]


def literal(value: float) -> str:
    text = repr(value)
    mantissa, _, exponent = text.partition("e")
    sign = "-" if mantissa.startswith("-") else ""
    mantissa = mantissa.lstrip("-")
    integer, _, fraction = mantissa.partition(".")
    grouped = "_".join(fraction[i : i + 3] for i in range(0, len(fraction), 3))
    out = f"{sign}{integer}.{grouped or '0'}"
    return f"{out}e{exponent}" if exponent else out


def main() -> None:
    print("const ASINPI_CELLS: Align64<[[f64; 8]; 16]> = Align64([")
    for function, domain in CELLS:
        coefficients = fit(function, domain)
        assert len(coefficients) == 8
        print(f"    // {function} on {domain}")
        print("    [")
        for coefficient in coefficients:
            print(f"        {literal(coefficient)},")
        print("    ],")
    print("]);")


if __name__ == "__main__":
    main()
