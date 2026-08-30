#!/usr/bin/env bash
# Sync CORE-MATH's hard-to-round (worst-case) corpora into tests/cases/.
#
# metallic reproduces CORE-MATH's `--worst` check step: each f64 function is
# gated bit-exact against the `core-math` oracle on the exact inputs CORE-MATH
# itself found hardest to round.  Those inputs are the `<fn>.wc` files shipped in
# the CORE-MATH source tree.  This script copies them in; the corpora are
# committed to git but excluded from the published crate (see `exclude` in
# Cargo.toml), so the dependency stays small while the repo stays self-checking.
#
# f32 functions are certified by exhaustive 2^32 sweeps instead, so only the
# bivariate ones (where a sweep is infeasible) carry a binary32 corpus.
#
# Usage:
#   tools/sync-worst-cases.sh                 # default checkout ~/src/core-math-sys
#   CORE_MATH_SYS=/path/to/core-math-sys tools/sync-worst-cases.sh
#
# Re-run after bumping the pinned `core-math` version to refresh the corpora.

set -euo pipefail

CORE_MATH_SYS="${CORE_MATH_SYS:-$HOME/src/core-math-sys}"
SRC="$CORE_MATH_SYS/vendor/src/binary64"
SRC128="$CORE_MATH_SYS/vendor/src/binary128"
DEST="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/tests/cases"

if [ ! -d "$SRC" ]; then
    echo "error: CORE-MATH binary64 sources not found at $SRC" >&2
    echo "       set CORE_MATH_SYS to your core-math-sys checkout." >&2
    exit 1
fi

# f64 functions metallic implements that have a CORE-MATH binary64 corpus.  The
# CORE-MATH directory name matches the function name for all of these.
FUNCS=(
    acos asin atan atan2 cbrt cos cosh acosh asinh atanh
    erf erfc exp exp2 exp10 expm1 hypot lgamma log log10
    log1p log2 pow sin sincos sinh tan tanh tgamma
    acospi asinpi atan2pi atanpi cospi sinpi tanpi
    exp2m1 exp10m1 log2p1 log10p1 rsqrt
)

# Bivariate f32 functions (exhaustive sweeps are infeasible there), as
# "<binary32 dir> <corpus file>" pairs — the dir is the C function name, the
# file carries the f-suffixed name.
SRC32="$CORE_MATH_SYS/vendor/src/binary32"
FUNCS32=(
    "atan2 atan2f"
    "atan2pi atan2pif"
    "compound compoundf"
    "hypot hypotf"
    "pow powf"
)

# binary128 pilots. Their q-suffixed corpus names match metallic's public API.
FUNCS128=(sqrt rsqrt cbrt hypot exp exp2 exp10 expm1 log asin acos atan atan2)

mkdir -p "$DEST"
missing=0
for f in "${FUNCS[@]}"; do
    wc="$SRC/$f/$f.wc"
    if [ -f "$wc" ]; then
        cp "$wc" "$DEST/$f.wc"
        printf '  %-8s %8s KB\n' "$f" "$(du -k "$DEST/$f.wc" | cut -f1)"
    else
        echo "  $f: MISSING ($wc)" >&2
        missing=$((missing + 1))
    fi
done

for pair in "${FUNCS32[@]}"; do
    read -r dir f <<< "$pair"
    wc="$SRC32/$dir/$f.wc"
    if [ -f "$wc" ]; then
        cp "$wc" "$DEST/$f.wc"
        printf '  %-8s %8s KB\n' "$f" "$(du -k "$DEST/$f.wc" | cut -f1)"
    else
        echo "  $f: MISSING ($wc)" >&2
        missing=$((missing + 1))
    fi
done

for f in "${FUNCS128[@]}"; do
    wc="$SRC128/$f/${f}q.wc"
    if [ -f "$wc" ]; then
        cp "$wc" "$DEST/${f}q.wc"
        printf '  %-8s %8s KB\n' "${f}q" "$(du -k "$DEST/${f}q.wc" | cut -f1)"
    else
        echo "  ${f}q: MISSING ($wc)" >&2
        missing=$((missing + 1))
    fi
done

total=$((${#FUNCS[@]} + ${#FUNCS32[@]} + ${#FUNCS128[@]}))
echo "synced $((total - missing))/$total corpora into $DEST"
[ "$missing" -eq 0 ]
