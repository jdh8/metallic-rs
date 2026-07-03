#!/usr/bin/env python3
"""Parse criterion estimates into a metallic-vs-CORE-MATH ratio table.

Reads every target/criterion/<id>/new/estimates.json, pairs each
`metallic::<fn>` with `core_math::<fn>`, and prints a grouped table
(slow >1.15, equal 0.95-1.15, fast <0.95) sorted by ratio.
"""
import json, glob, os, sys

CRIT = "target/criterion"
stat = sys.argv[1] if len(sys.argv) > 1 else "mean"  # "mean" or "median"

vals = {}  # id -> ns
for est in glob.glob(f"{CRIT}/*/new/estimates.json"):
    ident = os.path.basename(os.path.dirname(os.path.dirname(est)))
    with open(est) as f:
        d = json.load(f)
    vals[ident] = d[stat]["point_estimate"]

# group by function
fns = {}
for ident, ns in vals.items():
    if "__" in ident:
        lib, fn = ident.split("__", 1)
    else:
        continue
    fns.setdefault(fn, {})[lib] = ns

rows = []
for fn, d in fns.items():
    m = d.get("metallic")
    c = d.get("core_math")
    if m is None or c is None:
        continue
    rows.append((fn, m, c, m / c))

rows.sort(key=lambda r: -r[3])
print(f"# ratio = metallic / core_math  (stat={stat}, ns/op)\n")
print(f"{'fn':<8} {'metallic':>9} {'core_math':>9} {'ratio':>6}  group")
for fn, m, c, r in rows:
    grp = "SLOW" if r > 1.15 else ("fast" if r < 0.95 else "equal")
    print(f"{fn:<8} {m:>9.2f} {c:>9.2f} {r:>6.2f}  {grp}")

# also dump any fn missing a pair, for visibility
for fn, d in fns.items():
    if "metallic" not in d or "core_math" not in d:
        print(f"# unpaired: {fn} -> {sorted(d)}", file=sys.stderr)
