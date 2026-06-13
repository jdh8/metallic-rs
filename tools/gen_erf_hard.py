#!/usr/bin/env python3
"""Generate (and verify) the `ERFC_HARD` hard-to-round database for `f64::erfc`.

Run:  python3 tools/gen_erf_hard.py     (paste the array into src/f64_/erf.rs)

These are the residual inputs whose `erfc` lies closer to an f64 rounding
midpoint than the lifted double-double accurate path (`exp_dd_of_dd_accurate`,
≈2⁻¹⁰⁷) can resolve — found as the leftover mismatches on CORE-MATH's worst-case
corpus `tests/cases/erfc.wc` after the accurate-path lift (see
`examples/count_erf_wc.rs`).  Each input's correctly-rounded `erfc` is computed
here at 200-bit precision (mpmath), independently of CORE-MATH, then rounded once
to f64; the emitted `(input_bits, result_bits)` array is sorted for binary search.

`erf` needs no such table: once `ERF_TABLE_ZIV_EPS` is sound (≥ the table leg's
true 2⁻⁶¹·⁸ error) every hard `erf` case falls through to the correctly-rounded
accurate path.
"""
import struct
from mpmath import mp, mpf, erfc

mp.prec = 200

# Residual hard-to-round inputs (bit patterns), discovered on erfc.wc.
INPUTS = [
    0x3FE6317520FBE477,  # x ≈ 0.6935   (t-bridge segment 0)
    0x3FF76BD4D0E5284C,  # x ≈ 1.4638   (t-bridge segment 1)
    0xBFFF9A4A209CA0E4,  # x ≈ -1.9752  (2 − erfc reflection)
    0x4008CA123C6EED7E,  # x ≈ 3.0987   (t-bridge segment 2)
    0x4031D41CB671CAD3,  # x ≈ 17.829   (1/x² far leg, reduction-limited)
]


def bits_to_f64(b):
    return struct.unpack('<d', struct.pack('<Q', b))[0]


def f64_to_bits(x):
    return struct.unpack('<Q', struct.pack('<d', x))[0]


def cr_erfc(x):
    """Correctly-rounded f64 of erfc(x): 200-bit erfc, then round once to f64."""
    return float(erfc(mpf(x)))


rows = []
for b in INPUTS:
    x = bits_to_f64(b)
    want = cr_erfc(x)
    rows.append((b, f64_to_bits(want)))

rows.sort(key=lambda r: r[0])

print(f"const ERFC_HARD: [(u64, u64); {len(rows)}] = [")
for i in range(0, len(rows), 2):
    pair = rows[i:i + 2]
    line = "    " + " ".join(f"(0x{a:016x}, 0x{r:016x})," for a, r in pair)
    print(line)
print("];")
