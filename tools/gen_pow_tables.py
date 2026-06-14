#!/usr/bin/env python3
"""Generate `src/f64_/pow_consts.rs` from CORE-MATH's pow `dint.h` and `qint.h`.

This is a *verbatim transcription* tool: it parses the hex fields of the C
`dint64_t` / `qint64_t` struct literals and re-encodes them as Rust `Dint` /
`Qint` literals.  It does NOT recompute any numeric value.

A `dint64_t` is `{.hi, .lo, .ex, .sgn}` representing a 128-bit mantissa
`m = (hi << 64) | lo`.  The Rust `Dint { sgn: bool, ex: i64, m: u128 }` stores
that mantissa directly.

A `qint64_t` is `{.hh, .hl, .lh, .ll, .ex, .sgn}` with four 64-bit limbs.
The Rust `Qint { sgn: bool, ex: i64, hi: u128, lo: u128 }` stores
`hi = (hh << 64) | hl` and `lo = (lh << 64) | ll`.
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
DINT_H = "/home/jdh8/src/core-math-sys/vendor/src/binary64/pow/dint.h"
QINT_H = "/home/jdh8/src/core-math-sys/vendor/src/binary64/pow/qint.h"
OUT = os.path.join(ROOT, "src", "f64_", "pow_consts.rs")

# Hex field: 0x followed by hex digits.  ex: optional minus then decimal.
_HEX = r"0x[0-9a-fA-F]+"


def find_decl(text, c_name):
    """Locate `static const <type> <c_name>[opt-[]] = <init> ;`.

    Returns the substring of the initializer between the first '{' and its
    matching '}', i.e. the whole initializer body (with the outer braces).
    Works for both a scalar struct and an array of structs.
    """
    # Match the declaration head.  The name must be a whole identifier, and may
    # be followed by optional whitespace then `[]` for arrays, then `=`.
    pat = re.compile(
        r"static\s+const\s+(?:dint64_t|qint64_t)\s+"
        + re.escape(c_name)
        + r"\s*(?:\[\s*\])?\s*="
    )
    m = pat.search(text)
    if m is None:
        raise ValueError(f"could not find declaration of {c_name}")
    # Scan from the '=' to the first '{', then brace-match to its partner.
    i = m.end()
    n = len(text)
    while i < n and text[i] != "{":
        i += 1
    if i >= n:
        raise ValueError(f"no opening brace for {c_name}")
    start = i
    depth = 0
    while i < n:
        ch = text[i]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                # The terminating ';' should follow (possibly after spaces).
                j = i + 1
                while j < n and text[j] in " \t\r\n":
                    j += 1
                if j >= n or text[j] != ";":
                    raise ValueError(
                        f"declaration of {c_name} not terminated by ';'"
                    )
                return text[start : i + 1]
        i += 1
    raise ValueError(f"unbalanced braces for {c_name}")


def inner_groups(body):
    """Yield the brace groups inside `body` that contain a `.hi`/`.hh` field.

    For a scalar, `body` is itself `{ ... }`; for an array, `body` is
    `{ {..}, {..}, ... }`.  We find every brace-balanced `{...}` substring at
    the appropriate nesting and keep those that look like a struct literal
    (i.e. contain a `.hi` or `.hh` field).  This naturally ignores the array's
    outer braces (which contain no field at their own level).
    """
    n = len(body)
    i = 0
    groups = []
    while i < n:
        if body[i] == "{":
            depth = 0
            start = i
            while i < n:
                if body[i] == "{":
                    depth += 1
                elif body[i] == "}":
                    depth -= 1
                    if depth == 0:
                        groups.append(body[start : i + 1])
                        break
                i += 1
        i += 1
    # Keep only innermost struct literals.  If a captured group still contains
    # nested `{...}` (the array's outer wrapper), recurse into it.
    result = []
    for g in groups:
        inner = g[1:-1]
        if "{" in inner:
            result.extend(inner_groups(inner))
        elif ".hi" in g or ".hh" in g:
            result.append(g)
    return result


def field_hex(group, name):
    """Return the integer value of `.<name> = 0x...`, or 0 if absent."""
    m = re.search(r"\." + re.escape(name) + r"\s*=\s*(" + _HEX + r")", group)
    if m is None:
        return 0
    return int(m.group(1), 16)


def field_ex(group):
    m = re.search(r"\.ex\s*=\s*(-?\d+)", group)
    if m is None:
        return 0
    return int(m.group(1))


def parse_dint(group):
    hi = field_hex(group, "hi")
    lo = field_hex(group, "lo")
    sgn = field_hex(group, "sgn")
    ex = field_ex(group)
    m = (hi << 64) | lo
    return {"sgn": sgn != 0, "ex": ex, "m": m}


def parse_qint(group):
    hh = field_hex(group, "hh")
    hl = field_hex(group, "hl")
    lh = field_hex(group, "lh")
    ll = field_hex(group, "ll")
    sgn = field_hex(group, "sgn")
    ex = field_ex(group)
    hi = (hh << 64) | hl
    lo = (lh << 64) | ll
    return {"sgn": sgn != 0, "ex": ex, "hi": hi, "lo": lo}


def fmt_dint(d):
    return (
        f"Dint {{ sgn: {str(d['sgn']).lower()}, ex: {d['ex']}, "
        f"m: 0x{d['m']:032x} }}"
    )


def fmt_qint(q):
    return (
        f"Qint {{ sgn: {str(q['sgn']).lower()}, ex: {q['ex']}, "
        f"hi: 0x{q['hi']:032x}, lo: 0x{q['lo']:032x} }}"
    )


def parse_items(text, c_name, kind):
    """Parse all struct literals under declaration `c_name`.  Returns a list."""
    body = find_decl(text, c_name)
    groups = inner_groups(body)
    parse = parse_dint if kind == "dint" else parse_qint
    return [parse(g) for g in groups]


# (rust_name, c_name, kind, expected_count_or_None_for_scalar)
DINT_SCALARS = [
    ("ONE_D", "ONE"),
    ("M_ONE_D", "M_ONE"),
    ("LOG2_D", "LOG2"),
    ("LOG2_INV_D", "LOG2_INV"),
    ("ZERO_D", "ZERO"),
]
DINT_ARRAYS = [
    ("INVERSE_2_1", "_INVERSE_2_1", 92),
    ("INVERSE_2_2", "_INVERSE_2_2", 129),
    ("LOG_INV_2_1", "_LOG_INV_2_1", 92),
    ("LOG_INV_2_2", "_LOG_INV_2_2", 129),
    ("T1_2", "T1_2", 64),
    ("T2_2", "T2_2", 64),
    ("P_2", "P_2", 9),
    ("Q_2", "Q_2", 8),
]
QINT_SCALARS = [
    ("ONE_Q", "ONE_Q"),
    ("M_ONE_Q", "M_ONE_Q"),
    ("LOG2_Q", "LOG2_Q"),
    ("LOG2_INV_Q", "LOG2_INV_Q"),
    ("ZERO_Q", "ZERO_Q"),
]
QINT_ARRAYS = [
    ("INVERSE_3_1", "_INVERSE_3_1", 92),
    ("INVERSE_3_2", "_INVERSE_3_2", 129),
    ("LOG_INV_3_1", "_LOG_INV_3_1", 92),
    ("LOG_INV_3_2", "_LOG_INV_3_2", 129),
    ("T1_3", "T1_3", 64),
    ("T2_3", "T2_3", 64),
    ("P_3", "P_3", 18),
    ("Q_3", "Q_3", 15),
]

HEADER = """\
//! Accurate-path constant tables for f64 `pow`, transcribed verbatim from
//! CORE-MATH's `binary64/pow/dint.h` (Dint tables) and `qint.h` (Qint tables)
//! by `tools/gen_pow_tables.py`.  Do not edit by hand; re-run the generator.
//!
//! A few entries (e.g. `ONE_D`/`ONE_Q`) are transcribed for completeness though
//! the metallic accurate path does not reference them, hence `dead_code`.
#![allow(clippy::unreadable_literal)]
#![allow(dead_code)]

use super::dint::Dint;
use super::qint::Qint;
"""


def main():
    with open(DINT_H, "r") as f:
        dint_text = f.read()
    with open(QINT_H, "r") as f:
        qint_text = f.read()

    out = [HEADER]
    counts = []  # (name, got, expected)

    # ---- dint scalars ----
    for rust_name, c_name in DINT_SCALARS:
        items = parse_items(dint_text, c_name, "dint")
        assert len(items) == 1, f"{c_name}: expected scalar, got {len(items)}"
        out.append(f"pub const {rust_name}: Dint = {fmt_dint(items[0])};\n")

    # ---- dint arrays ----
    for rust_name, c_name, expected in DINT_ARRAYS:
        items = parse_items(dint_text, c_name, "dint")
        counts.append((rust_name, len(items), expected))
        assert len(items) == expected, (
            f"{c_name}: expected {expected} entries, got {len(items)}"
        )
        out.append(f"pub const {rust_name}: [Dint; {len(items)}] = [")
        for it in items:
            out.append(f"    {fmt_dint(it)},")
        out.append("];\n")

    # ---- qint scalars ----
    for rust_name, c_name in QINT_SCALARS:
        items = parse_items(qint_text, c_name, "qint")
        assert len(items) == 1, f"{c_name}: expected scalar, got {len(items)}"
        out.append(f"pub const {rust_name}: Qint = {fmt_qint(items[0])};\n")

    # ---- qint arrays ----
    for rust_name, c_name, expected in QINT_ARRAYS:
        items = parse_items(qint_text, c_name, "qint")
        counts.append((rust_name, len(items), expected))
        assert len(items) == expected, (
            f"{c_name}: expected {expected} entries, got {len(items)}"
        )
        out.append(f"pub const {rust_name}: [Qint; {len(items)}] = [")
        for it in items:
            out.append(f"    {fmt_qint(it)},")
        out.append("];\n")

    with open(OUT, "w") as f:
        f.write("\n".join(out))
        f.write("\n")

    # ---------------- report ----------------
    print(f"wrote {OUT}")
    print()
    print("array entry counts (got / expected):")
    all_ok = True
    for name, got, expected in counts:
        ok = "OK" if got == expected else "MISMATCH"
        if got != expected:
            all_ok = False
        print(f"  {name:<14} {got:>4} / {expected:<4}  {ok}")
    print()

    # ---------------- validation ----------------
    print("validation literals:")
    log2_d = parse_items(dint_text, "LOG2", "dint")[0]
    log2_q = parse_items(qint_text, "LOG2_Q", "qint")[0]
    one_d = parse_items(dint_text, "ONE", "dint")[0]
    inv21 = parse_items(dint_text, "_INVERSE_2_1", "dint")
    inv31 = parse_items(qint_text, "_INVERSE_3_1", "qint")
    print(f"  LOG2_D         = {fmt_dint(log2_d)}")
    print(f"  LOG2_Q         = {fmt_qint(log2_q)}")
    print(f"  ONE_D          = {fmt_dint(one_d)}")
    print(f"  INVERSE_2_1[0] = {fmt_dint(inv21[0])}")
    print(f"  INVERSE_3_1[0] = {fmt_qint(inv31[0])}")

    expected_lits = {
        "LOG2_D": "Dint { sgn: false, ex: -1, m: 0xb17217f7d1cf79abc9e3b39803f2f6af }",
        "LOG2_Q": "Qint { sgn: false, ex: -1, hi: 0xb17217f7d1cf79abc9e3b39803f2f6af, lo: 0x40f343267298b62d8a0d175b8baafa2b }",
        "ONE_D": "Dint { sgn: false, ex: 0, m: 0x80000000000000000000000000000000 }",
        "INVERSE_2_1[0]": "Dint { sgn: false, ex: 0, m: 0xb5000000000000000000000000000000 }",
        "INVERSE_3_1[0]": "Qint { sgn: false, ex: 0, hi: 0xb5000000000000000000000000000000, lo: 0x00000000000000000000000000000000 }",
    }
    print()
    print("validation checks:")
    got_lits = {
        "LOG2_D": fmt_dint(log2_d),
        "LOG2_Q": fmt_qint(log2_q),
        "ONE_D": fmt_dint(one_d),
        "INVERSE_2_1[0]": fmt_dint(inv21[0]),
        "INVERSE_3_1[0]": fmt_qint(inv31[0]),
    }
    for k, want in expected_lits.items():
        got = got_lits[k]
        ok = got == want
        if not ok:
            all_ok = False
        print(f"  {k:<14} {'OK' if ok else 'FAIL'}")
        if not ok:
            print(f"    expected: {want}")
            print(f"    got     : {got}")

    if not all_ok:
        sys.exit(1)


if __name__ == "__main__":
    main()
