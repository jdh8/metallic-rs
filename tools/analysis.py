#!/usr/bin/env python3
"""Generate ANALYSIS.md: static, per-function cost estimates beside CORE-MATH.

    python3 tools/analysis.py all

`asm` walks the fast path and the accurate leg out of the emitted assembly at
each ISA level, `mca` prices them with llvm-mca, `legs` counts how many finite
inputs reach the accurate leg through LLVM source coverage, `calibrate` records
one Criterion median per precision, and `render` writes the report from the
artefacts under `analysis/` alone.  Python 3.9+, standard library only; needs
`clang`, `gdb`, `nm`, `llvm-objdump`, `llvm-mca`, `llvm-profdata` and `llvm-cov`
on PATH (matching LLVM majors for rustc and the coverage tools) and, unless
`--no-f128`, a nightly toolchain for the binary128 functions.
"""

import argparse
import bisect
import filecmp
import json
import os
import re
import subprocess
import tempfile
import threading
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import date
from functools import lru_cache
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TRIPLE = "x86_64-unknown-linux-gnu"
# One pass through the block on an empty pipeline: the latency of one call.
# `-iterations=1000` (the block chained through its own result) reads the
# same dependent latency on the Intel and znver2 models, but the znver3/znver4
# register-file models drop the loop-carried dependency whenever a zero idiom
# (`vxorps %xmm0, %xmm0, %xmm0`, the `return 0.0` arm) hits the source of an
# eliminated move — the prologue of most Rust functions — and report about a
# quarter of the real figure (metallic `exp` on znver4: 17 against 70 cycles).
ITERATIONS = 1
SHORT_PATH = 20

ISA = {
    "x86_64": {
        # level: (rustc -Ctarget-cpu / TARGET_CPU, clang -march, llvm-mca -mcpu)
        "levels": {
            "v2": ("x86-64-v2", "x86-64-v2", "x86-64-v2"),
            "v3": ("x86-64-v3", "x86-64-v3", "x86-64-v3"),
            "v4": ("x86-64-v4", "x86-64-v4", "x86-64-v4"),
            "native": ("native", "native", None),   # None: detect (see native_model)
        },
        "cycles": ("v3", "v4", "native"),   # columns priced by llvm-mca
        "fma_count": "v2",                   # column that counts FMA calls instead
        # LLVM spells the 64-bit forms `jmpq` (every indirect jump) and `callq`.
        "cond": r"^j(?!mp[lq]?\b)[a-z]+\b",  # conditional branch mnemonics
        "jmp": r"^jmp[lq]?\b", "ret": r"^ret", "call": r"^call",
        # Call targets that are FMA thunks: `fma`, `fmaf`, `fmaq`, `fmaf128`,
        # `force_fma`, the mangled `3fma` segment — not libm's `fmax`/`fmaxf`.
        "fma": r"fma(?!x)",
    },
}

# Public functions, in report order; the suffix does not tell `erf` from `erff`.
FUNCTIONS = {
    "f32": """acosf acoshf acospif asinf asinhf asinpif atan2f atan2pif atanf atanhf
        atanpif cbrtf compoundf cosf coshf cospif erfcf erff exp2f exp2m1f exp10f exp10m1f
        expf expm1f fmaf frexpf hypotf ldexpf lgammaf log1pf log2f log2p1f log10f log10p1f
        logf powf roundf rsqrtf sincosf sinf sinhf sinpif tanf tanhf tanpif tgammaf""".split(),
    "f64": """acos acosh acospi asin asinh asinpi atan atan2 atan2pi atanh atanpi cbrt
        compound cos cosh cospi erf erfc exp exp2 exp2m1 exp10 exp10m1 expm1 fma frexp
        hypot ldexp lgamma log log1p log2 log2p1 log10 log10p1 pow round rsqrt sin sincos
        sinh sinpi tan tanh tanpi tgamma""".split(),
    "f128": """acosq asinq atan2q atanq cbrtq cosq exp2q exp10q expm1q expq hypotq log1pq
        log2q log10q logq powq rsqrtq sinq sqrtq tanq""".split(),
}
# Functions the `core_math` crate does not bind (`analysis list` prints 0).
NO_CORE_MATH = set("""compound fma fmaf frexp frexpf ldexp ldexpf round roundf
    cosq sinq tanq log2q log10q log1pq powq""".split())
CALIBRATION = ("expf", "exp", "expq")

# The argument whose instructions are the fast path: `analysis call <fn> x y z`
# is traced under gdb on both sides and the report prices what that call
# executed.  Keyed by the f64 name (the f32/f128 twins share it); `y` is the
# second operand, `z` the third of `fma`, `ldexp` reads its exponent from `y`.
INPUT_DEFAULT = (1.7, 0.7, 0.3)
INPUT = {
    **{n: (0.7, 0.7, 0.3) for n in ("acos", "asin", "atanh", "acospi", "asinpi", "erf", "erfc")},
    "tgamma": (4.5, 0.7, 0.3), "lgamma": (4.5, 0.7, 0.3),
    "ldexp": (1.7, 3.0, 0.3),
}
# `legs` sample counts per bucket when neither a flag nor a stored count
# applies: f32 univariate exhaustive (0), 2³⁰ for f64 and for the f32
# bivariate/trivariate probes (`--f64-samples` serves both), 2²⁸ for f128.
DEFAULT_SAMPLES = {"f32": 0, "f32_multi": 2**30, "f64": 2**30, "f128": 2**28}

# One entry per public function: "m" is metallic, "c" CORE-MATH (None where
# the `core_math` crate has no binding).  `cov`: coverage anchors
# "<file path suffix>: <unique source substring>" summed for the accurate-leg
# count (anchor the leg's definition line or the first statement inside the
# fallback block; an `if` needs the explicit BRDA form below;
# empty list = no accurate leg). `leg`:
# "sym:<ident>" (walk from the label containing `<len><ident>` for Rust — a
# `::` path such as "exp::accurate" concatenates the segments, for legs whose
# bare name recurs in other modules, and "log::accurate<Natural>" picks one
# generic instantiation — or equal to <ident> for C),
# "branch:<k>" (inline leg: walk from the side of the k-th conditional branch
# on the traced fast path that the trace did not go — its target, or its
# fall-through when the trace took it; negative counts from the end; the
# `branches` list in the side's JSON shows them in order), or None.
# `branch:je:-1` first filters by mnemonic, keeping a gate distinct from
# later sign branches that can disappear at another ISA level.
# `prec`: free text. A `cov` entry may be a dict with `source` plus an
# `offset` to a nearby line, or `branch: [block, edge]` for a precise LCOV
# BRDA count (C gates before loops or early returns). Sum disjoint entries,
# never both the parent accurate tier and its descendant.
# `acc_take` (optional): conditional-branch indices or mnemonics the accurate leg's static
# walk takes instead of falling through — the k-th conditional branch in the
# written `<fn>.acc.s`, counted across walked-into calls — when its
# fall-through runs on into a further tier or an early return.
# BEGIN FN
FN = {
    "acosf": {
        "m": {"cov": ["f32_/atan.rs: fn acosf_fallback(x: f32) -> f32 {"], "leg": "sym:acosf_fallback", "prec": "f64 → f64 + exceptions", "acc_take": ["jne"]},
        "c": {"cov": ["binary32/acos/acosf.c: if(t.u == 0x328885a3u)", "binary32/acos/acosf.c: double bx = __builtin_fabs(xs);"], "leg": "branch:-1", "prec": "f64 → f64 + exceptions", "acc_take": [2]},
    },
    "acoshf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/acosh/acoshf.c: double c0 = cp[0] + z*cp[1];"], "leg": "branch:-1", "prec": "f64 → f64"},
    },
    "acospif": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "asinf": {
        "m": {"cov": ["f32_/atan.rs: fn asinf_fallback(x: f32) -> f32 {"], "leg": "sym:asinf_fallback", "prec": "f64 → f64 + exceptions"},
        "c": {"cov": ["binary32/asin/asinf.c: double z = xs, z2 = z*z, c0 = poly12(z2, c);", "binary32/asin/asinf.c: if(__builtin_expect(ax == 0x7e55688au, 0))"], "leg": "branch:-1", "prec": "f64 → f64 + exceptions"},
    },
    "asinhf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/asinh/asinhf.c: double c0 = cp[0] + z*cp[1];"], "leg": "branch:-1", "prec": "f64 → f64"},
    },
    "asinpif": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "atan2f": {
        "m": {"cov": ["f32_/atan.rs: fn atan2_mag_accurate(a: f64, b: f64, x_negative: bool) -> f32 {"], "leg": "sym:atan2_mag_accurate", "prec": "f64 → double-double"},
        "c": {"cov": ["binary32/atan2/atan2f.c: double dy = y, dx = x;", "binary32/atan2/atan2f.c: double z2l, z2h = muldd(zh,zl,zh,zl,&z2l);"], "leg": "branch:-1", "prec": "f64 → f64 / double-double"},
    },
    "atan2pif": {
        "m": {"cov": ["f32_/atan.rs: fn atan2pi_mag_accurate(a: f64, b: f64, x_negative: bool) -> f32 {"], "leg": "sym:atan2pi_mag_accurate", "prec": "f64 → double-double"},
        "c": {"cov": ["binary32/atan2pi/atan2pif.c: double z2l, z2h = muldd(zh,zl,zh,zl,&z2l);"], "leg": "branch:-1", "prec": "f64 → double-double"},
    },
    "atanf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "atanhf": {
        "m": {"cov": ["f32_/hyp.rs: fn atanhf_fallback(x: f32) -> f32 {"], "leg": "sym:atanhf_fallback", "prec": "f64 → f64"},
        "c": {"cov": ["binary32/atanh/atanhf.c: double zn4 = zn2*zn2, zd4 = zd2*zd2;"], "leg": "branch:-1", "prec": "f64 → f64"},
    },
    "atanpif": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "cbrtf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/cbrt/cbrtf.c: double h = f*f*f - z;"], "leg": "branch:-1", "prec": "f64 → f64"},
    },
    "compoundf": {
        "m": {"cov": ["f32_/pow.rs: fn exact_compound(x: f64, y: f64) -> Option<f32> {"], "leg": "branch:-1", "prec": "f64 → exact integer / double-double"},
        "c": {"cov": ["binary32/compound/compoundf.c: log2p1_accurate (&h, &l, x);"], "leg": "sym:accurate_path", "prec": "f64 → double-double"},
    },
    "cosf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/cos/cosf.c: static float __attribute__((noinline)) as_cosf_database(float x, double r){"], "leg": "sym:as_cosf_database", "prec": "f64 → f32 exception table"},
    },
    "coshf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/cosh/coshf.c: h = (iln2h*z - ia) + iln2l*z;"], "leg": "branch:-1", "prec": "f64 → f64"},
    },
    "cospif": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "erfcf": {
        "m": {"cov": ["f32_/erf.rs: fn erfc_dd(x: f64) -> DoubleDouble {"], "leg": "sym:erfc_dd", "prec": "f64 → double-double"},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "erff": {
        "m": {"cov": ["f32_/erf.rs: fn erfc_dd(x: f64) -> DoubleDouble {"], "leg": "sym:erfc_dd", "prec": "f64 → double-double"},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "exp2f": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/exp2/exp2f.c: if(__builtin_expect(ux<=0x79e7526eu, 0)){"], "leg": "branch:2", "prec": "f64 → f64 + exceptions"},
    },
    "exp2m1f": {
        "m": {"cov": ["f32_/exp.rs: crate::f64_::exp2m1(x) as f32"], "leg": "sym:exp2m1", "prec": "f64 → double-double + exceptions → 128-bit", "acc_take": [3]},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "exp10f": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/exp10/exp10f.c: h = (iln102h*z - ia*0.03125) + iln102l*z;"], "leg": "branch:2", "prec": "f64 → f64"},
    },
    "exp10m1f": {
        "m": {"cov": ["f32_/exp.rs: return f32::from_bits(0x59c6_4405);", "f32_/exp.rs: crate::f64_::exp10m1(x) as f32"], "leg": "sym:exp10m1", "prec": "f64 → double-double + exceptions → 128-bit", "acc_take": [3]},
        "c": {"cov": [], "leg": None, "prec": "f64 + exact-case table"},
    },
    "expf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/exp/expf.c: h = (iln2h*z + ia) + iln2l*z;"], "leg": "branch:1", "prec": "f64 → f64"},
    },
    "expm1f": {
        "m": {"cov": ["f32_/exp.rs: let r = crate::fast_mul_add(n, -LN_2_HI, x);"], "leg": "branch:-1", "prec": "f64 → f64"},
        "c": {"cov": ["binary32/expm1/expm1f.c: if(__builtin_expect(ux>0xc18aa123u, 0)) // x < -17.32"], "leg": "branch:2", "prec": "f64 → f64"},
    },
    "fmaf": {
        "m": {"cov": [], "leg": None, "prec": "f32 FMA"},
        "c": None,
    },
    "frexpf": {
        "m": {"cov": [], "leg": None, "prec": "32-bit integer"},
        "c": None,
    },
    "hypotf": {
        "m": {"cov": ["f32_/misc.rs: let candidate_f64 = f64::from(candidate);"], "leg": "branch:4", "prec": "f64 → double-double", "acc_take": [0]},
        "c": {"cov": ["binary32/hypot/hypotf.c: double cd = c;"], "leg": "branch:-1", "prec": "f64 → double-double"},
    },
    "ldexpf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": None,
    },
    "lgammaf": {
        "m": {"cov": ["f32_/gamma.rs: fn lgamma_dd(z: f32) -> f32 {"], "leg": "sym:f32_::gamma::lgamma_dd", "prec": "f64 → double-double"},
        "c": {"cov": ["binary32/lgamma/lgammaf.c: int a = 0, b = sizeof(tb)/sizeof(tb[0]);"], "leg": "branch:-1", "prec": "f64 → f32 exception table"},
    },
    "log1pf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/log1p/log1pf.c: double z4 = z2*z2, f = z2*((b[1] + z*b[2])"], "leg": "branch:-1", "prec": "f64 → f64 pairs"},
    },
    "log2f": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/log2/log2f.c: double c0 = c[0] + z*c[1];"], "leg": "branch:-1", "prec": "f64 → f64"},
    },
    "log2p1f": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/log2p1/log2p1f.c: double c0 = c[0] + d*c[1];"], "leg": "branch:-1", "prec": "f64 → f64 + exceptions"},
    },
    "log10f": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/log10/log10f.c: double f = z*((c[0] + z*c[1]) + z2*((c[2] + z*c[3]) + z2*(c[4] + z*c[5] + z2*c[6])));"], "leg": "branch:-1", "prec": "f64 → f64"},
    },
    "log10p1f": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": [{"source": "binary32/log10p1/log10p1f.c: if(__builtin_expect(ub != lb, 0)){", "branch": [0, 0]}], "leg": "branch:-1", "prec": "f64 → f64 + exceptions", "acc_take": [0]},
    },
    "logf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/log/logf.c: double f = z2*((c[0] + z*c[1]) + z2*((c[2] + z*c[3]) + z2*(c[4] + z*c[5] + z2*c[6])));"], "leg": "branch:-1", "prec": "f64 → f64"},
    },
    "powf": {
        "m": {"cov": ["f32_/pow.rs: exp2_dd(log2_dd(x) * y)"], "leg": "branch:-1", "prec": "f64 → double-double"},
        "c": {"cov": ["binary32/pow/powf.c: static float as_powf_accurate2(float x0, float y0, int is_exact, FLAG_T flag){"], "leg": "branch:8", "prec": "f64 → double-double"},
    },
    "roundf": {
        "m": {"cov": [], "leg": None, "prec": "f32"},
        "c": None,
    },
    "rsqrtf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "sincosf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/sincos/sincosf.c: static void __attribute__((noinline)) as_sincosf_database(float x, float *sout, float *cout){"], "leg": "sym:as_sincosf_database", "prec": "f64 → f32 exception table"},
    },
    "sinf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "sinhf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/sinh/sinhf.c: h = (iln2h*z - ia) + iln2l*z;"], "leg": "branch:-1", "prec": "f64 → f64"},
    },
    "sinpif": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "tanf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": ["binary32/tan/tanf.c:   uint32_t ax = t.u&(~0u>>1), sgn = t.u>>31;"], "leg": "branch:-1", "prec": "f64 → f32 exception table"},
    },
    "tanhf": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "tanpif": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": {"cov": [], "leg": None, "prec": "f64"},
    },
    "tgammaf": {
        "m": {"cov": ["f32_/gamma.rs: fn tgamma_dd(x: f64) -> f32 {"], "leg": "sym:tgamma_dd", "prec": "f64 → double-double"},
        "c": {"cov": [{"source": "binary32/tgamma/tgammaf.c: if(__builtin_expect(((rt.u+2)&0xfffffff) < 8, 0)){", "branch": [0, 0]}, {"source": "binary32/tgamma/tgammaf.c: if(((rt.u+2)&0xfffffff) < 4){", "branch": [0, 0]}], "leg": "branch:-1", "prec": "f64 → f32 exception table"},
    },
    "acos": {
        "m": {"cov": ["f64_/atan.rs: fn acos_accurate(x: f64) -> f64 {"], "leg": "sym:acos_accurate", "prec": "double-double → double-double → triple-double"},
        "c": {"cov": ["binary64/acos/acos.c: double as_acos_refine(double x, double phi){"], "leg": "sym:as_acos_refine", "prec": "double-double → double-double + exceptions"},
    },
    "acosh": {
        "m": {"cov": ["f64_/hyp.rs: fn ln_sqrt_accurate(x: f64, c: DoubleDouble, d: DoubleDouble) -> f64 {", "f64_/hyp.rs: super::dint::ln_2s_corrected_accurate(s, correction)"], "leg": "sym:ln_sqrt_accurate", "prec": "double-double → 128-bit"},
        "c": {"cov": ["binary64/acosh/acosh.c: static double __attribute__((noinline)) as_acosh_one(double x, double sh, double sl){", "binary64/acosh/acosh.c: static double as_acosh_refine(double x, double a){"], "leg": "sym:as_acosh_refine", "prec": "f64 / double-double → double-double / triple-double + exceptions"},
    },
    "acospi": {
        "m": {"cov": ["f64_/atan.rs: fn acospi_accurate(x: f64) -> f64 {"], "leg": "sym:acospi_accurate", "prec": "double-double → double-double → triple-double"},
        "c": {"cov": ["binary64/acospi/acospi.c: double absx, y, h, l, u, v;"], "leg": "sym:accurate_path", "prec": "double-double → double-double + exceptions"},
    },
    "asin": {
        "m": {"cov": ["f64_/atan.rs: fn asin_accurate(a: f64) -> f64 {"], "leg": "sym:asin_accurate", "prec": "double-double → double-double → triple-double"},
        "c": {"cov": ["binary64/asin/asin.c: double as_asin_refine(double x, double phi){"], "leg": "sym:as_asin_refine", "prec": "double-double → double-double + exceptions"},
    },
    "asinh": {
        "m": {"cov": ["f64_/hyp.rs: fn asinh_small_accurate(x: f64) -> f64 {", "f64_/hyp.rs: fn ln_sqrt_accurate(x: f64, c: DoubleDouble, d: DoubleDouble) -> f64 {", "f64_/hyp.rs: super::dint::ln_2s_corrected_accurate(s, correction)"], "leg": "sym:ln_sqrt_accurate", "prec": "f64 / double-double → double-double / 128-bit"},
        "c": {"cov": ["binary64/asinh/asinh.c: static double __attribute__((noinline)) as_asinh_zero(double x, double x2h, double x2l){", "binary64/asinh/asinh.c: static double as_asinh_refine(double x, double zh, double zl, double a){"], "leg": "sym:as_asinh_refine", "prec": "f64 / double-double → double-double / triple-double + exceptions"},
    },
    "asinpi": {
        "m": {"cov": ["f64_/atan.rs: fn asinpi_accurate(a: f64) -> f64 {", "f64_/atan.rs: fn asinpi_tiny_accurate(x: f64) -> f64 {"], "leg": "sym:asinpi_accurate", "prec": "double-double → double-double → triple-double / 128-bit"},
        "c": {"cov": ["binary64/asinpi/asinpi.c: static double asinpi_acc(double x){"], "leg": "sym:asinpi_acc", "prec": "64/128-bit fixed → 128-bit fixed"},
    },
    "atan": {
        "m": {"cov": ["f64_/atan.rs: fn atan_accurate(a: f64) -> f64 {"], "leg": "sym:atan_accurate", "prec": "f64 → double-double → 128-bit"},
        "c": {"cov": ["binary64/atan/atan.c: static double __attribute__((cold,noinline)) as_atan_refine2(double x, double a){"], "leg": "sym:as_atan_refine2", "prec": "f64 → double-double + exceptions"},
    },
    "atan2": {
        "m": {"cov": ["f64_/atan.rs: let inner = if q < 9.094947017729282e-13 {", "f64_/atan.rs: return atan2_tint_mag(a, b, x_negative);"], "leg": "branch:-1", "prec": "f64 → double-double → 192-bit"},
        "c": {"cov": ["binary64/atan2/atan2.c: feholdexcept(&env);"], "leg": "sym:atan2_accurate", "prec": "f64 pair → double-double → 192-bit"},
    },
    "atan2pi": {
        "m": {"cov": ["f64_/atan.rs: let inner = if q < 9.094_947_017_729_282e-13 {", "f64_/atan.rs: return atan2pi_tint_mag(a, b, x_negative);"], "leg": "branch:-1", "prec": "f64 / double-double → double-double → 192-bit"},
        "c": {"cov": ["binary64/atan2pi/atan2pi.c: double absy = __builtin_fabs (y), absx = __builtin_fabs (x);"], "leg": "sym:atan2pi_accurate", "prec": "double-double → 192-bit", "acc_take": [0]},
    },
    "atanh": {
        "m": {"cov": ["f64_/hyp.rs: fn atanh_small_accurate(x: f64) -> f64 {", "f64_/hyp.rs: super::dint::ln_dd_scaled(u.high, u.low, -1)"], "leg": "branch:-1", "prec": "double-double → double-double / 128-bit"},
        "c": {"cov": ["binary64/atanh/atanh.c: static double __attribute__((noinline)) as_atanh_zero(double x){", "binary64/atanh/atanh.c: static double as_atanh_refine(double x, double zh, double zl, double a){"], "leg": "sym:as_atanh_refine", "prec": "double-double → double-double / triple-double + exceptions"},
    },
    "atanpi": {
        "m": {"cov": ["f64_/atan.rs: fn atanpi_accurate(a: f64) -> f64 {", "f64_/atan.rs: fn atanpi_tiny_accurate(x: f64) -> f64 {"], "leg": "sym:atanpi_accurate", "prec": "f64 / double-double → double-double → 128-bit"},
        "c": {"cov": ["binary64/atanpi/atanpi.c: static double __attribute__((noinline)) as_atan_refine2(double x, double a){"], "leg": "sym:as_atan_refine2", "prec": "f64 / double-double → double-double + exceptions"},
    },
    "cbrt": {
        "m": {"cov": ["f64_/misc.rs: let quotient = DoubleDouble::from_quotient(zz, y1) / y1; // zz / y1\u00b2"], "leg": "branch:-1", "prec": "f64 / double-double → double-double"},
        "c": {"cov": ["binary64/cbrt/cbrt.c: y2 = y1*y1; y2l = __builtin_fma(y1,y1,-y2);"], "leg": "branch:-2", "prec": "f64 / double-double → double-double + exceptions"},
    },
    "compound": {
        "m": {"cov": ["f64_/pow_accurate.rs: pub(super) fn compound_accurate(x: f64, y: f64) -> f64 {"], "leg": "sym:compound_accurate", "prec": "double-double → 128-bit → exact integer / 256-bit"},
        "c": None,
    },
    "cos": {
        "m": {"cov": ["f64_/trig.rs: let v = if q.wrapping_add(1) & 2 != 0 {"], "leg": "branch:-1", "prec": "f64 / double-double → 128-bit"},
        "c": {"cov": ["binary64/cos/cos.c: return cos_accurate (t.f);"], "leg": "sym:cos_accurate", "prec": "double-double → 128-bit + exceptions"},
    },
    "cosh": {
        "m": {"cov": ["f64_/hyp.rs: fn cosh_accurate(x: f64) -> f64 {"], "leg": "sym:cosh_accurate", "prec": "double-double → double-double + exceptions"},
        "c": {"cov": ["binary64/cosh/cosh.c: static double __attribute__((noinline)) as_cosh_zero(double x){", "binary64/cosh/cosh.c: th = as_exp_accurate(ax, t, th, tl, &tl);", "binary64/cosh/cosh.c: if(__builtin_expect(aix>0x403f666666666666ull, 0)){", "binary64/cosh/cosh.c: rl = ((th - rh) + qh) + ql + tl;"], "leg": "sym:as_exp_accurate", "prec": "f64 / double-double → double-double + exceptions"},
    },
    "cospi": {
        "m": {"cov": ["f64_/trig.rs: return cospi_accurate(x);", "f64_/trig.rs: fn cospi_dd(x: f64) -> DoubleDouble {"], "leg": "branch:3", "prec": "f64 / double-double → double-double → 128-bit"},
        "c": {"cov": ["binary64/cospi/cospi.c: static double as_cospi_zero(double x){", "binary64/cospi/cospi.c: static double as_sinpi_refine(int iq, double z){"], "leg": "sym:as_sinpi_refine", "prec": "f64 / double-double → double-double + exceptions"},
    },
    "erf": {
        "m": {"cov": ["f64_/erf.rs: fn erf_small_accurate(x: f64) -> f64 {", "f64_/erf.rs: let r = ONE + neg(scale_dd(m, q));"], "leg": "branch:-1", "prec": "double-double → double-double"},
        "c": {"cov": ["binary64/erf/erf.c: cr_erf_accurate (&h, &l, z);"], "leg": "branch:-1", "prec": "double-double → double-double + exceptions"},
    },
    "erfc": {
        "m": {"cov": ["f64_/erf.rs: fn database_lookup(table: &[(u64, u64)], key: u64) -> Option<f64> {"], "leg": "branch:-1", "prec": "double-double → double-double + exceptions"},
        "c": {"cov": ["binary64/erfc/erfc.c: return cr_erfc_accurate (x);"], "leg": "branch:-1", "prec": "double-double → double-double + exceptions", "acc_take": list(range(1, 18))},
    },
    "exp": {
        "m": {"cov": ["src/f64_/exp.rs: fn exp_accurate(x: f64) -> f64 {"], "leg": "sym:exp_accurate", "prec": "f64 pair → double-double + exceptions", "acc_take": [0, 1]},
        "c": {"cov": ["binary64/exp/exp.c: static double __attribute__((cold,noinline)) as_exp_accurate(double x){"], "leg": "sym:as_exp_accurate", "prec": "f64 pair → double-double + exceptions"},
    },
    "exp2": {
        "m": {"cov": ["src/f64_/exp.rs: fn exp2_accurate(x: f64) -> f64 {"], "leg": "sym:exp2_accurate", "prec": "f64 pair → double-double + exceptions", "acc_take": [0]},
        "c": {"cov": ["binary64/exp2/exp2.c: static double __attribute__((cold,noinline)) as_exp2_accurate(double x){"], "leg": "sym:as_exp2_accurate", "prec": "f64 pair → double-double + exceptions"},
    },
    "exp2m1": {
        "m": {"cov": ["src/f64_/exp.rs: fn exp2m1_general(x: f64) -> f64 {", "src/f64_/exp.rs: fn exp2m1_deep(x: f64, c: DoubleDouble) -> f64 {"], "leg": "sym:exp2m1_general", "prec": "f64 pair / double-double → double-double + exceptions / 128-bit", "acc_take": [0]},
        "c": {"cov": ["binary64/exp2m1/exp2m1.c: return exp2m1_accurate (x);"], "leg": "sym:exp2m1_accurate", "prec": "double-double → double-double + exceptions"},
    },
    "exp10": {
        "m": {"cov": ["src/f64_/exp.rs: fn exp10_accurate(x: f64) -> f64 {"], "leg": "sym:exp10_accurate", "prec": "f64 pair → double-double + exceptions", "acc_take": [0]},
        "c": {"cov": ["binary64/exp10/exp10.c: static double __attribute__((noinline)) as_exp10_accurate(double x){"], "leg": "sym:as_exp10_accurate", "prec": "f64 pair → double-double + exceptions"},
    },
    "exp10m1": {
        "m": {"cov": ["src/f64_/exp.rs: fn exp10m1_general(x: f64) -> f64 {", "src/f64_/exp.rs: fn exp10m1_deep(x: f64, c: DoubleDouble) -> f64 {"], "leg": "sym:exp10m1_general", "prec": "f64 pair / double-double → double-double + exceptions / 128-bit", "acc_take": [0]},
        "c": {"cov": ["binary64/exp10m1/exp10m1.c: return exp10m1_accurate (x);"], "leg": "sym:exp10m1_accurate", "prec": "double-double → double-double + exceptions"},
    },
    "expm1": {
        "m": {"cov": ["src/f64_/exp.rs: fn expm1_general(x: f64) -> f64 {"], "leg": "sym:expm1_general", "prec": "f64 pair / double-double → double-double + exceptions", "acc_take": [0]},
        "c": {"cov": ["binary64/expm1/expm1.c: static double __attribute__((noinline)) as_expm1_accurate(double x){"], "leg": "sym:as_expm1_accurate", "prec": "f64 pair → double-double + exceptions"},
    },
    "fma": {
        "m": {"cov": [], "leg": None, "prec": "f64 FMA"},
        "c": None,
    },
    "frexp": {
        "m": {"cov": [], "leg": None, "prec": "64-bit integer"},
        "c": None,
    },
    "hypot": {
        "m": {"cov": ["f64_/misc.rs: fn hypot_hard(big_s: f64, small_s: f64) -> f64 {"], "leg": "sym:hypot_hard", "prec": "f64 / double-double → exact 128-bit integer"},
        "c": {"cov": ["binary64/hypot/hypot.c: static double  __attribute__((noinline)) as_hypot_hard(double x, double y, const fexcept_t flag){"], "leg": "sym:as_hypot_hard", "prec": "f64 / double-double → exact 128-bit integer"},
    },
    "ldexp": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": None,
    },
    "lgamma": {
        "m": {"cov": [{"source": "f64_/gamma.rs: let (value, gate) = lgamma_mid(z);", "offset": 4}, "f64_/gamma.rs: fn lgamma_dd(z: f64) -> TripleDouble {"], "leg": "sym:f64_::gamma::lgamma_dd", "prec": "double-double → double-double → triple-double"},
        "c": {"cov": ["binary64/lgamma/lgamma.c: static __attribute__((noinline)) double as_lgamma_accurate(double x){"], "leg": "sym:as_lgamma_accurate", "prec": "double-double → triple-double + exceptions"},
    },
    "log": {
        "m": {"cov": ["f64_/dint.rs: pub fn ln_accurate(x: f64) -> f64 {"], "leg": "branch:-1", "prec": "f64 / double-double → 128-bit"},
        "c": {"cov": ["binary64/log/log.c: dint_fromd (&X, x);"], "leg": "branch:-1", "prec": "double-double → 128-bit"},
    },
    "log1p": {
        "m": {"cov": ["f64_/log.rs: fn log1p_small_accurate(x: f64) -> f64 {", "f64_/dint.rs: pub fn log1p_accurate(s: f64, c: f64) -> f64 {"], "leg": "sym:log1p_accurate", "prec": "f64 / double-double → double-double / 128-bit"},
        "c": {"cov": ["binary64/log1p/log1p.c: static double __attribute__((noinline)) as_log1p_refine(double x, double a){"], "leg": "sym:as_log1p_refine", "prec": "double-double → double-double / triple-double", "acc_take": [0]},
    },
    "log2": {
        "m": {"cov": ["f64_/dint.rs: pub fn log2_accurate(x: f64) -> f64 {"], "leg": "branch:-1", "prec": "f64 / double-double → 128-bit"},
        "c": {"cov": ["binary64/log2/log2.c: static double __attribute__((noinline)) as_log2_refine(double x, double a){"], "leg": "sym:as_log2_refine", "prec": "double-double → double-double / triple-double"},
    },
    "log2p1": {
        "m": {"cov": ["f64_/dint.rs: pub fn log2p1_deep(x: f64) -> f64 {", "f64_/dint.rs: pub fn log2p1_accurate(s: f64, c: f64) -> f64 {"], "leg": "sym:log2p1_accurate", "prec": "f64 / double-double → 128-bit"},
        "c": {"cov": ["binary64/log2p1/log2p1.c: double ax = __builtin_fabs (x);"], "leg": "branch:-1", "prec": "double-double → double-double + exceptions / 128-bit"},
    },
    "log10": {
        "m": {"cov": ["f64_/dint.rs: pub fn log10_accurate(x: f64) -> f64 {"], "leg": "branch:-1", "prec": "f64 / double-double → 128-bit"},
        "c": {"cov": ["binary64/log10/log10.c: dint_fromd (&X, x);"], "leg": "branch:-1", "prec": "double-double → 128-bit"},
    },
    "log10p1": {
        "m": {"cov": ["f64_/dint.rs: pub fn log10p1_deep(x: f64) -> f64 {", "f64_/dint.rs: pub fn log10p1_accurate(s: f64, c: f64) -> f64 {"], "leg": "sym:log10p1_accurate", "prec": "f64 / double-double → 128-bit"},
        "c": {"cov": ["binary64/log10p1/log10p1.c: double ax = __builtin_fabs (x);"], "leg": "sym:cr_log10p1_accurate", "prec": "double-double → double-double + exceptions / 128-bit"},
    },
    "pow": {
        "m": {"cov": ["f64_/pow_accurate.rs: pub(super) fn pow_accurate(x: f64, y: f64, x0: f64, s: f64) -> f64 {"], "leg": "branch:-1", "prec": "double-double → 128-bit → exact integer / 256-bit"},
        "c": {"cov": ["binary64/pow/pow.c: if (y == 1.0)"], "leg": "branch:-1", "prec": "double-double → 128-bit → exact integer / 256-bit", "acc_take": [0, 1, 2]},
    },
    "round": {
        "m": {"cov": [], "leg": None, "prec": "f64"},
        "c": None,
    },
    "rsqrt": {
        "m": {"cov": ["f64_/misc.rs: fn rsqrt_refine(rf: f64, x: f64) -> f64 {"], "leg": "sym:rsqrt_refine", "prec": "f64 / double-double → exact 128-bit integer", "acc_take": [1]},
        "c": {"cov": ["binary64/rsqrt/rsqrt.c: static double __attribute__((noinline)) as_rsqrt_refine(double rf, double a){"], "leg": "sym:as_rsqrt_refine", "prec": "f64 / double-double → exact 128-bit integer", "acc_take": [1]},
    },
    "sin": {
        "m": {"cov": ["f64_/trig.rs: let v = if q & 2 != 0 { neg_dint(v) } else { v };"], "leg": "branch:-2", "prec": "f64 / double-double → 128-bit"},
        "c": {"cov": ["binary64/sin/sin.c: return sin_accurate (x);"], "leg": "sym:sin_accurate", "prec": "double-double → 128-bit + exceptions"},
    },
    "sincos": {
        "m": {"cov": ["f64_/trig.rs: select_sin_dint(q, s, c).to_f64(),"], "leg": "branch:je:-1", "prec": "f64 / double-double → 128-bit"},
        "c": {"cov": ["binary64/sincos/sincos.c: if (*c == right_c) // fast path succeeded for cos"], "leg": "sym:sin_accurate", "prec": "double-double → 128-bit + exceptions"},
    },
    "sinh": {
        "m": {"cov": ["f64_/hyp.rs: fn sinh_small_accurate(x: f64) -> f64 {", "f64_/hyp.rs: fn sinh_accurate(x: f64) -> f64 {"], "leg": "sym:sinh_accurate", "prec": "f64 / double-double → double-double + exceptions"},
        "c": {"cov": ["binary64/sinh/sinh.c: static double __attribute__((noinline)) as_sinh_zero(double x){", "binary64/sinh/sinh.c: th = as_exp_accurate(ax, t, th, tl, &tl);", "binary64/sinh/sinh.c: aix>0x403f666666666666ull, 0)){ // |x| > 31.4", "binary64/sinh/sinh.c: rl = ((th-rh)-qh) - ql + tl;"], "leg": "sym:as_exp_accurate", "prec": "f64 / double-double → double-double + exceptions"},
    },
    "sinpi": {
        "m": {"cov": ["f64_/trig.rs: return sinpi_accurate(x);", "f64_/trig.rs: fn sinpi_dd(x: f64) -> DoubleDouble {"], "leg": "sym:sinpi_dd", "prec": "f64 / double-double → double-double → 128-bit"},
        "c": {"cov": ["binary64/sinpi/sinpi.c: static double as_sinpi_zero(double x){", "binary64/sinpi/sinpi.c: static double as_sinpi_refine(int iq, double z){"], "leg": "sym:as_sinpi_refine", "prec": "f64 / double-double → double-double + exceptions"},
    },
    "tan": {
        "m": {"cov": ["f64_/trig.rs: let num = select_sin_dint(q, s, c);"], "leg": "branch:-2", "prec": "double-double → 128-bit"},
        "c": {"cov": ["binary64/tan/tan.c: return tan_accurate (x);"], "leg": "sym:tan_accurate", "prec": "double-double → 128-bit + exceptions"},
    },
    "tanh": {
        "m": {"cov": ["f64_/hyp.rs: fn tanh_small_accurate(x: f64) -> f64 {", "f64_/hyp.rs: fn tanh_accurate(x: f64) -> f64 {"], "leg": "sym:tanh_accurate", "prec": "f64 / double-double → double-double + exceptions"},
        "c": {"cov": ["binary64/tanh/tanh.c: static double __attribute__((noinline)) as_tanh_zero(double x){ // |x|<0.25", "binary64/tanh/tanh.c: double rl, rh = as_exp_accurate(-2*ax, t, th, tl, &rl);"], "leg": "sym:as_exp_accurate", "prec": "f64 / double-double → double-double + exceptions"},
    },
    "tanpi": {
        "m": {"cov": ["f64_/trig.rs: fn tanpi_accurate(x: f64) -> f64 {"], "leg": "sym:tanpi_accurate", "prec": "double-double → 128-bit"},
        "c": {"cov": ["binary64/tanpi/tanpi.c: z *= 0x1p-63;", "binary64/tanpi/tanpi.c: double dx2 = __builtin_fma(x,x,-x2), dx3 = __builtin_fma(x2,x,-x3) + dx2*x, dv;"], "leg": "branch:4", "prec": "double-double → double-double + exceptions"},
    },
    "tgamma": {
        "m": {"cov": ["f64_/gamma.rs: fn tgamma_accurate(z: f64) -> f64 {"], "leg": "sym:tgamma_accurate", "prec": "double-double → triple-double"},
        "c": {"cov": ["binary64/tgamma/tgamma.c: static __attribute__((noinline)) double as_tgamma_accurate(double x){"], "leg": "sym:as_tgamma_accurate", "prec": "double-double → double-double + exceptions"},
    },
    "acosq": {
        "m": {"cov": ["f128_/asin.rs: fn accurate(fx: u128, ex: i32, sq: &Sqrt, acos: bool, xneg: bool, sign: u128) -> f128 {"], "leg": "sym:asin::accurate", "prec": "128-bit fixed (256-bit frame) → 384-bit fixed"},
        "c": {"cov": ["binary128/acos/acosq.c: __float128 as_acosq_accurate(__float128 x){"], "leg": "sym:as_acosq_accurate", "prec": "128-bit fixed (192-bit frame) → 320-bit fixed"},
    },
    "asinq": {
        "m": {"cov": ["f128_/asin.rs: fn accurate(fx: u128, ex: i32, sq: &Sqrt, acos: bool, xneg: bool, sign: u128) -> f128 {"], "leg": "sym:asin::accurate", "prec": "128-bit fixed (256-bit frame) → 384-bit fixed"},
        "c": {"cov": ["binary128/asin/asinq.c: __float128 as_asinq_accurate(__float128 x){"], "leg": "sym:as_asinq_accurate", "prec": "128-bit fixed (192/256-bit frame) → 320-bit fixed"},
    },
    "atan2q": {
        "m": {"cov": ["f128_/atan2.rs: fn accurate(r: &Reduction, sign: u128) -> f128 {"], "leg": "sym:atan2::accurate", "prec": "128-bit fixed (256-bit frame) → 384-bit fixed"},
        "c": {"cov": ["binary128/atan2/atan2q.c: __float128 as_atan2_accurate(__float128 y, __float128 x){"], "leg": "sym:as_atan2_accurate", "prec": "128-bit fixed (192-bit frame) → 384-bit fixed"},
    },
    "atanq": {
        "m": {"cov": ["f128_/atan2.rs: fn accurate_exact(t1: u128, et: i32, sign: u128) -> f128 {", "f128_/atan2.rs: fn accurate(r: &Reduction, sign: u128) -> f128 {"], "leg": "sym:atan2::accurate", "prec": "128-bit fixed (256-bit frame) → 384-bit fixed"},
        "c": {"cov": ["binary128/atan/atanq.c: __float128 as_atanq_accurate(__float128 x){"], "leg": "sym:as_atanq_accurate", "prec": "128-bit fixed (192-bit frame) → 384-bit fixed"},
    },
    "cbrtq": {
        "m": {"cov": ["f128_/roots.rs: fn correct_cbrt(mantissa: u128, remainder: i32, mut candidate: f128) -> f128 {"], "leg": "branch:2", "prec": "128-bit fixed → exact 384-bit integer"},
        "c": {"cov": ["binary128/cbrt/cbrtq.c: u128 c = (sx1.a + (1<<14))>>15 | (u128)1<<113;"], "leg": "branch:3", "prec": "128-bit fixed → 256-bit / modular 128-bit integer"},
    },
    "cosq": {
        "m": {"cov": ["f128_/trig.rs: fn accurate(m: u128, e: i32, cosine: bool, sign: u128) -> f128 {"], "leg": "sym:trig::accurate", "prec": "128-bit fixed (256-bit frame) → 384-bit fixed"},
        "c": None,
    },
    "exp2q": {
        "m": {"cov": ["f128_/exp.rs: fn accurate(m: u128, e: i32, negative: bool, l: &Reduction) -> f128 {"], "leg": "sym:exp::accurate", "prec": "128-bit fixed → 256-bit fixed"},
        "c": {"cov": ["binary128/exp2/exp2q.c: static void __attribute__((noinline)) as_exp2q_accurate(int *el, u2x64 m, u128 x0){"], "leg": "sym:as_exp2q_accurate", "prec": "128-bit fixed → 192-bit fixed → 384-bit fixed"},
    },
    "exp10q": {
        "m": {"cov": ["f128_/exp.rs: fn accurate(m: u128, e: i32, negative: bool, l: &Reduction) -> f128 {"], "leg": "sym:exp::accurate", "prec": "128-bit fixed → 256-bit fixed"},
        "c": {"cov": ["binary128/exp10/exp10q.c: static void __attribute__((noinline)) as_exp10q_accurate(int *el, u2x64 m, u128 x0){"], "leg": "sym:as_exp10q_accurate", "prec": "128-bit fixed → 192-bit fixed → 384-bit fixed"},
    },
    "expm1q": {
        "m": {"cov": ["f128_/exp.rs: fn expm1_accurate(m: u128, e: i32, negative: bool) -> f128 {", "f128_/exp.rs: fn expm1_small(m: u128, e: i32, y: [u128; 3], negative: bool) -> f128 {"], "leg": "sym:expm1_accurate", "prec": "128-bit fixed → 256-bit fixed (384-bit product)"},
        "c": {"cov": ["binary128/expm1/expm1q.c: static void __attribute__((noinline)) as_expm1q_accurate(int *el, u2x64 m, u128 x0){"], "leg": "sym:as_expm1q_accurate", "prec": "128-bit fixed → 192-bit fixed → 384-bit fixed"},
    },
    "expq": {
        "m": {"cov": ["f128_/exp.rs: fn accurate(m: u128, e: i32, negative: bool, l: &Reduction) -> f128 {"], "leg": "sym:exp::accurate", "prec": "128-bit fixed → 256-bit fixed"},
        "c": {"cov": ["binary128/exp/expq.c: static void __attribute__((noinline)) as_expq_accurate(int *el, u2x64 m, u128 x0){"], "leg": "sym:as_expq_accurate", "prec": "128-bit fixed → 192-bit fixed → 384-bit fixed", "acc_take": [3]},
    },
    "hypotq": {
        "m": {"cov": ["f128_/hypot.rs: fn exact(ma: u128, mb: u128, dn: u32, eb: i32) -> f128 {"], "leg": "branch:6", "prec": "128-bit fixed → exact 384-bit integer", "acc_take": [4]},
        "c": {"cov": ["binary128/hypot/hypotq.c: v.a += 1<<13;"], "leg": "branch:4", "prec": "128-bit fixed → 256-bit / modular 128-bit integer"},
    },
    "log1pq": {
        "m": {"cov": ["f128_/log.rs: fn accurate<B: Base>(e: i32, j: u32, d: [u128; 3]) -> f128 {", "f128_/log.rs: fn small_accurate(m: u128, e: i32, negative: bool) -> f128 {"], "leg": "sym:log::accurate<Natural>", "prec": "128/256-bit fixed → 256/384-bit fixed"},
        "c": None,
    },
    "log2q": {
        "m": {"cov": ["f128_/log.rs: fn accurate<B: Base>(e: i32, j: u32, d: [u128; 3]) -> f128 {"], "leg": "sym:log::accurate<Binary>", "prec": "256-bit fixed → 384-bit fixed"},
        "c": None,
    },
    "log10q": {
        "m": {"cov": ["f128_/log.rs: fn accurate<B: Base>(e: i32, j: u32, d: [u128; 3]) -> f128 {"], "leg": "sym:log::accurate<Decimal>", "prec": "256-bit fixed → 384-bit fixed"},
        "c": None,
    },
    "logq": {
        "m": {"cov": ["f128_/log.rs: fn accurate<B: Base>(e: i32, j: u32, d: [u128; 3]) -> f128 {"], "leg": "sym:log::accurate<Natural>", "prec": "256-bit fixed → 384-bit fixed"},
        "c": {"cov": ["binary128/log/logq.c: static u64 __attribute__((noinline)) as_logq_refine(i64 el, u2x64 m, __float128 x0){"], "leg": "sym:as_logq_refine", "prec": "128-bit fixed (192-bit frame) → 384-bit fixed (448-bit product)"},
    },
    "powq": {
        "m": {"cov": ["f128_/pow.rs: fn accurate("], "leg": "sym:pow::accurate", "prec": "128/256-bit fixed → 256/384-bit fixed → exact integer / 640-bit fixed"},
        "c": None,
    },
    "rsqrtq": {
        "m": {"cov": ["f128_/roots.rs: fn correct_rsqrt(mantissa: u128, exponent: i32, mut candidate: f128) -> f128 {"], "leg": "branch:6", "prec": "128-bit fixed → exact 384-bit integer"},
        "c": {"cov": ["binary128/rsqrt/rsqrtq.c: v.a += 1<<13;"], "leg": "branch:4", "prec": "128-bit fixed → exact 256-bit integer"},
    },
    "sinq": {
        "m": {"cov": ["f128_/trig.rs: fn accurate(m: u128, e: i32, cosine: bool, sign: u128) -> f128 {"], "leg": "sym:trig::accurate", "prec": "128-bit fixed (256-bit frame) → 384-bit fixed"},
        "c": None,
    },
    "sqrtq": {
        "m": {"cov": ["f128_/roots.rs: fn correct_sqrt(mantissa: u128, exponent: i32, mut candidate: f128) -> f128 {"], "leg": "branch:5", "prec": "128-bit fixed → exact 256/384-bit integer"},
        "c": {"cov": ["binary128/sqrt/sqrtq.c: v.a += 1<<13;"], "leg": "branch:3", "prec": "128-bit fixed → modular 128-bit integer"},
    },
    "tanq": {
        "m": {"cov": ["f128_/tan.rs: fn accurate(m: u128, e: i32, sign: u128) -> f128 {"], "leg": "sym:tan::accurate", "prec": "128-bit fixed (256-bit frame) → 384-bit fixed"},
        "c": None,
    },
}
# END FN

EMPTY_SIDE = {"cov": [], "leg": None, "prec": "?"}
VERBOSE = False


# --- Process helpers -------------------------------------------------------

def fail(message):
    print("analysis: " + message, file=sys.stderr)
    sys.exit(2)


def run(cmd, env=None, cwd=None, capture=True, check=True, stdin=None):
    """Run `cmd` with CC=clang set; return the CompletedProcess.

    `capture`: True captures both streams, False neither, "stdout" only
    stdout (stderr passes through, for cargo's rendered diagnostics)."""
    full = os.environ.copy()
    full["CC"] = "clang"
    full.update(env or {})
    if VERBOSE:
        prefix = " ".join(k + "=" + v for k, v in sorted((env or {}).items()))
        print("+ " + (prefix + " " if prefix else "") + " ".join(map(str, cmd)), file=sys.stderr)
    streams = {"capture_output": True} if capture is True else \
        {"stdout": subprocess.PIPE} if capture == "stdout" else {}
    return subprocess.run([str(c) for c in cmd], env=full, cwd=str(cwd or ROOT),
                          text=True, check=check, input=stdin, **streams)


def cargo(args, target_dir, env, f128, artifacts=False):
    """`cargo [+nightly] <args> [--features f128] --target x86_64-unknown-linux-gnu`.

    With `artifacts`, cargo reports every unit it compiled or found fresh as
    a JSON line (diagnostics still rendered on stderr), and the parsed
    `compiler-artifact` messages come back, each naming its output files."""
    args = list(args)
    # Cargo's own options go before a `--` (what follows it is for rustc).
    split = args.index("--") if "--" in args else len(args)
    options = (["--features", "f128"] if f128 else []) + ["--target", TRIPLE]
    if artifacts:
        options.append("--message-format=json-render-diagnostics")
    cmd = ["cargo"] + (["+nightly"] if f128 else []) + args[:split] + options + args[split:]
    full = dict(env)
    full["CARGO_TARGET_DIR"] = str(target_dir)
    out = run(cmd, env=full, capture="stdout" if artifacts else False)
    if not artifacts:
        return None
    messages = (json.loads(line) for line in out.stdout.splitlines() if line.startswith("{"))
    return [m for m in messages if m.get("reason") == "compiler-artifact"]


def core_math_sys():
    """Directory of the vendored core-math-sys crate, from `cargo metadata`."""
    out = run(["cargo", "metadata", "--format-version", "1", "--offline"]).stdout
    for package in json.loads(out)["packages"]:
        if package["name"] == "core-math-sys":
            return Path(package["manifest_path"]).parent
    fail("core-math-sys not found in cargo metadata")


@lru_cache(maxsize=None)
def native_model():
    """clang's -target-cpu for -march=native, the llvm-mca model of this host."""
    out = run(["clang", "-march=native", "-###", "-c", "-x", "c", "/dev/null"],
              check=False)
    match = re.search(r'-target-cpu"?\s+"?([\w-]+)', out.stderr + out.stdout)
    if not match:
        fail("cannot detect the native CPU model from clang -### output")
    return match.group(1)


def mca_model(level):
    model = ISA["x86_64"]["levels"][level][2]
    return native_model() if model is None else model


def rustc_version(f128):
    return run(["rustc"] + (["+nightly"] if f128 else []) + ["--version"]).stdout.strip()


def clang_version():
    return run(["clang", "--version"]).stdout.splitlines()[0].strip()


def c_source(sys_dir, name, prec):
    """Vendored CORE-MATH source of a public function: binary64/exp/exp.c, binary32/exp/expf.c."""
    base = name[:-1] if prec in ("f32", "f128") else name
    width = {"f32": "binary32", "f64": "binary64", "f128": "binary128"}[prec]
    return sys_dir / "vendor" / "src" / width / base / (name + ".c")


def precision_of(name):
    for prec, names in FUNCTIONS.items():
        if name in names:
            return prec
    fail("unknown function " + name)


def input_of(name):
    """The traced argument of `name`, shared with its f32/f128 twins."""
    known = {n for names in FUNCTIONS.values() for n in names}
    base = name[:-1] if name[-1] in "fq" and name[:-1] in known else name
    return INPUT.get(base, INPUT_DEFAULT)


def side_of(name, key):
    """FN entry for one side, tolerant of missing entries; None when CORE-MATH has no binding."""
    if key == "c" and name in NO_CORE_MATH:
        return None
    side = FN.get(name, {}).get(key, EMPTY_SIDE)
    if side is None:
        return None
    return {"cov": side.get("cov", []), "leg": side.get("leg"), "prec": side.get("prec", "?"),
            "acc_take": side.get("acc_take", [])}


def selected(only, f128):
    """Ordered (name, prec) pairs after --only and --no-f128."""
    names = [(n, p) for p in ("f32", "f64", "f128") if f128 or p != "f128"
             for n in FUNCTIONS[p]]
    if only:
        wanted = set(only)
        unknown = wanted - {n for n, _ in names}
        if unknown:
            fail("unknown or excluded functions: " + ", ".join(sorted(unknown)))
        names = [(n, p) for n, p in names if n in wanted]
    return names


# --- Assembly ---------------------------------------------------------------

LABEL = re.compile(r"^([A-Za-z_.$][\w.$]*):")
DATA_SIZES = {".byte": 1, ".short": 2, ".value": 2, ".2byte": 2, ".hword": 2,
              ".long": 4, ".4byte": 4, ".int": 4, ".quad": 8, ".8byte": 8, ".octa": 16}
SECTION = {".section", ".text", ".data", ".bss", ".rodata"}
MEMREF = re.compile(r"(?:^|[\s,*])([A-Za-z_.][\w.$@]*)(?:[+-]\d+)?\(")


def strip_comment(line):
    """Drop a trailing `#` comment, leaving `#` inside double quotes alone."""
    quoted, i = False, 0
    while i < len(line):
        ch = line[i]
        if ch == "\\" and quoted:
            i += 2
            continue
        if ch == '"':
            quoted = not quoted
        elif ch == "#" and not quoted:
            return line[:i]
        i += 1
    return line


def split_operands(text):
    """Split directive operands on commas outside double quotes."""
    parts, depth, current, quoted = [], 0, [], False
    prev = ""
    for ch in text:
        if ch == '"' and prev != "\\":
            quoted = not quoted
        if ch == "," and not quoted and depth == 0:
            parts.append("".join(current).strip())
            current = []
        else:
            if ch == "(" and not quoted:
                depth += 1
            elif ch == ")" and not quoted:
                depth -= 1
            current.append(ch)
        prev = ch
    if current:
        parts.append("".join(current).strip())
    return [p for p in parts if p]


def string_bytes(literal):
    """Byte length of a GNU as string literal (octal, hex and single-char escapes)."""
    body = literal.strip()
    if body.startswith('"') and body.endswith('"'):
        body = body[1:-1]
    count, i = 0, 0
    while i < len(body):
        if body[i] == "\\":
            i += 1
            if i < len(body) and body[i] in "01234567":
                j = i
                while j < len(body) and j - i < 3 and body[j] in "01234567":
                    j += 1
                i = j
            elif i < len(body) and body[i] == "x":
                i += 1
                while i < len(body) and body[i] in "0123456789abcdefABCDEF":
                    i += 1
            else:
                i += 1
        else:
            i += 1
        count += 1
    return count


def directive_bytes(directive, operands):
    """Bytes emitted by one data directive, or None for a non-data directive."""
    if directive in DATA_SIZES:
        return DATA_SIZES[directive] * len(split_operands(operands))
    if directive in (".zero", ".space", ".skip"):
        size = split_operands(operands)[0]
        return int(size, 0)
    if directive == ".ascii":
        return sum(string_bytes(s) for s in split_operands(operands))
    if directive in (".asciz", ".string"):
        return sum(string_bytes(s) + 1 for s in split_operands(operands))
    return None


def directive_items(directive, operands):
    """One data directive as comparable `(size, operand)` items: one per
    operand for the fixed-size directives, one for the whole of a `.zero`,
    `.ascii`, or the like (whose spelling is its content)."""
    if directive in DATA_SIZES:
        return [(DATA_SIZES[directive], o) for o in split_operands(operands)]
    return [(directive, operands.strip())]


class Program:
    """One parsed `.s` file: instructions, label positions and data sizes.

    `insns[i] == (mnemonic, operands)`; `labels[name]` is the index of the
    first instruction at or after the label; `data[name]` is the number of
    bytes of data directives between the label and the next label, section
    directive or instruction, and `content[name]` those directives as a tuple
    of `(size, operand)` items (one per operand, so `.quad a, b` and two
    `.quad`s compare equal) — the key that tells two labels with the same
    bytes apart from two with different ones.
    """

    def __init__(self, text, name="<asm>"):
        self.name = name
        self.insns = []
        self.labels = {}
        self.data = {}
        self.content = {}
        current = None
        for raw in text.splitlines():
            line = strip_comment(raw).strip()
            if not line:
                continue
            label = LABEL.match(line)
            if label:
                current = label.group(1)
                self.labels[current] = len(self.insns)
                self.data[current] = 0
                self.content[current] = []
                rest = line[label.end():].strip()
                if not rest:
                    continue
                line = rest
            if line.startswith("."):
                parts = line.split(None, 1)
                directive = parts[0]
                operands = parts[1] if len(parts) > 1 else ""
                if directive in SECTION:
                    current = None
                    continue
                size = directive_bytes(directive, operands)
                if size is not None and current is not None:
                    self.data[current] += size
                    self.content[current] += directive_items(directive, operands)
                continue
            parts = line.split(None, 1)
            self.insns.append((parts[0], parts[1].strip() if len(parts) > 1 else ""))
            current = None
        self.content = {name: tuple(items) for name, items in self.content.items()}


class Side:
    """The `.s` files of one side; local `.L` labels resolve per file, others across files."""

    def __init__(self, programs):
        self.programs = list(programs)

    @staticmethod
    def load(paths):
        return Side(Program(Path(p).read_text(), str(p)) for p in paths)

    def resolve(self, label, home):
        """(program, index) of a branch target, or None when it is not defined here."""
        label = label.split("@")[0]
        order = [home] + [p for p in self.programs if p is not home]
        if label.startswith(".L"):
            order = [home]
        for program in order:
            if label in program.labels:
                return program, program.labels[label]
        return None

    def find(self, predicate):
        """Every (program, label) whose label satisfies `predicate`."""
        return [(p, l) for p in self.programs for l in p.labels if predicate(l)]

    def data_of(self, program, symbol):
        """(bytes, content) of a data symbol: a `.L` label in `program` alone
        (assembler-local names recur across files with different contents),
        any other across the side's files; (0, ()) when it is not data here."""
        symbol = symbol.split("@")[0]
        order = [program] if symbol.startswith(".L") else self.programs
        for candidate in order:
            if symbol in candidate.data:
                return candidate.data[symbol], candidate.content[symbol]
        return 0, ()


def symbol_matcher(leg):
    """Predicate on label/call-target names for a `sym:` leg.

    `sym:a::b` matches a Rust label containing `1a1b`-style `<len><ident>`
    segments back to back (legacy and v0 mangling both keep them), `sym:a::b<T>`
    one whose generic argument `1T` follows, or a C label equal to the last
    segment (optionally `@PLT`).
    """
    spec = leg.split(":", 1)[1]
    generic = None
    if spec.endswith(">"):
        spec, generic = spec[:-1].split("<", 1)
    path = spec.split("::")
    pattern = r"(?<!\d)" + "".join(str(len(p)) + re.escape(p) for p in path)
    if generic:   # v0 spells the argument `<len><Type>` after the path
        pattern += r".*?(?<!\d)" + str(len(generic)) + re.escape(generic)
    mangled = re.compile(pattern)
    plain = path[-1]

    def match(name):
        bare = name.split("@")[0]
        if bare.startswith("_RNC"):   # v0 closure nested inside the named function
            return False
        return bare == plain or bool(mangled.search(name))
    return match


GOT_LOAD = re.compile(r"^(?:movq|leaq)\s+([\w.$]+)@GOTPCREL\(%rip\),\s*(%\w+)$")
SYMBOL = re.compile(r"[\w.$]+")
MAX_DEPTH = 8


def call_target(operands, loaded):
    """The symbol a `call` or `jmp` operand names: `sym`, `sym@PLT`,
    `*sym@GOTPCREL(%rip)`, or `*%reg` after a `movq sym@GOTPCREL(%rip), %reg`
    on the path (`loaded`).  An unknown register or a jump table
    (`*.LJTI0_0(,%rax,8)`) comes back as written, which `SYMBOL` rejects."""
    if operands.startswith("*%"):
        return loaded.get(operands[1:], operands)
    target = operands.lstrip("*")
    return target.split("@")[0] if "@" in target else target


def walk(side, program, index, take=()):
    """Static straight-line walk from `program.insns[index]`: the accurate leg.

    Conditional branches fall through unless their encounter index is in `take`;
    `jmp` to a local `.L` label is followed; `ret` stops; `ud2` stops with
    `trap`; a revisited instruction is a loop's back edge (`loop`): its body
    counts once and the walk leaves through the latest conditional branch of
    the same descent whose other side is still unvisited, or stops.  A `take`
    index beyond the branches met flags `take`.  A `call` whose target is
    defined in this side's own assembly is walked into and its `ret` returns
    to the caller, as if inlined — except FMA thunks (the ISA `fma` regex),
    which stay calls; so does every other unresolved target (`@PLT`, GOT
    indirection to another object, an unknown register).  Those remaining
    calls are recorded and stepped over (`call`). An LLVM `.LJTI` jump
    table selects its first arm (`dispatch`); other indirect jumps stop.
    A `jmp` to anything but a `.L` label is a tail call and gets the same treatment: a target defined
    here is followed (`tail`), any other is recorded as a call and, since it
    returns to the walked function's caller, ends the walk (or the descent).
    Opaque tail jumps become `callq` for llvm-mca's flat call cost (`opaque-tail`);
    an unrecognized indirect target stops with `indirect`. Returns a dict
    with `insns` (list of "mnemonic\\toperands"), `calls`, `fma_calls`,
    `flags` (sorted list), `symbols` (memory-referenced (program, symbol)
    pairs — `.L` labels are file-local) and `branches` (per conditional
    branch, in encounter order across descents: (program, target label,
    fall-through index, taken)).
    """
    # ponytail: one static accurate path; trace forced-fallback cases if band-specific costs are needed.
    cfg = ISA["x86_64"]
    cond, jmp, ret, call, fma = (re.compile(cfg[k]) for k in ("cond", "jmp", "ret", "call", "fma"))
    insns, calls, flags, symbols, branches = [], [], set(), set(), []
    fma_calls = 0
    visited = set()
    frames = []      # (program, return index, caller's visited) per descent
    owners = []      # keep each branch's visited set alive across later calls
    loaded = {}      # register -> symbol of the last `movq sym@GOTPCREL(%rip), %reg`
    dispatch = None  # (program, .LJTI symbol, instruction) within this basic block
    pos = (program, index)
    while True:
        program, i = pos
        if i >= len(program.insns):
            flags.add("end")
            break
        if (id(program), i) in visited:
            flags.add("loop")
            pos = None
            for k in reversed(range(len(branches))):
                b_program, target, fallthrough, taken = branches[k]
                if owners[k] is not visited or not target.startswith(".L"):
                    continue
                other = (b_program, fallthrough) if taken else side.resolve(target, b_program)
                if other is not None and (id(other[0]), other[1]) not in visited:
                    pos = other
                    break
            if pos is None:
                break
            continue
        visited.add((id(program), i))
        mnemonic, operands = program.insns[i]
        text = mnemonic + ("\t" + operands if operands else "")
        for symbol in MEMREF.findall(operands):
            symbols.add((program, symbol.split("@")[0]))
            if symbol.startswith(".LJTI"):
                dispatch = (program, symbol, i)
        got = GOT_LOAD.match(text)
        if got:
            loaded[got.group(2)] = got.group(1)
        elif operands.endswith(tuple(loaded)) and "," in operands:
            loaded.pop(operands.rsplit(",", 1)[1].strip(), None)
        if cond.match(mnemonic):
            dispatch = None
            insns.append(text)
            k = len(branches)
            taken = k in take or mnemonic in take
            branches.append((program, operands, i + 1, taken))
            owners.append(visited)
            if taken:
                dest = side.resolve(operands, program)
                if not operands.startswith(".L"):
                    flags.add("tail")
                if dest is None:
                    break
                pos = dest
            else:
                pos = (program, i + 1)
        elif jmp.match(mnemonic):
            insns.append(text)
            if operands.startswith(".L"):
                dispatch = None
                dest = side.resolve(operands, program)
                if dest is None:
                    break
                pos = dest
                continue
            # A tail call: `sym`, `sym@PLT`, `*sym@GOTPCREL(%rip)`, a register
            # loaded from the GOT — or a jump table / unknown register.
            target = call_target(operands, loaded)
            if not SYMBOL.fullmatch(target):
                # Native scheduling interleaves floating-point work with
                # leaq/movslq/addq/jmp. Verify the integer address chain,
                # rather than imposing an instruction-distance limit.
                if dispatch and dispatch[0] is program:
                    table = dispatch[1]
                    scalar = [(m, o) for m, o in program.insns[dispatch[2]:i]
                              if not m.startswith("v")]
                    prepared = operands.startswith("*" + table + "(")
                    if len(scalar) >= 3 and scalar[0][0] == "leaq":
                        base = re.fullmatch(re.escape(table) + r"\(%rip\),\s*(%\w+)", scalar[0][1])
                        load = re.fullmatch(r"\(" + re.escape(base[1]) + r",%\w+,4\),\s*(%\w+)",
                                            scalar[-2][1]) if base else None
                        prepared |= bool(load and scalar[-2][0] == "movslq"
                                         and scalar[-1] == ("addq", base[1] + ", " + load[1])
                                         and operands == "*" + load[1])
                    entries = program.content.get(table, ())
                    first = re.fullmatch(r"(\.L[\w.$]+)(?:-" + re.escape(table) + r")?",
                                         entries[0][1]) if prepared and entries else None
                    dest = side.resolve(first[1], program) if first else None
                    if dest is not None:
                        flags.add("dispatch")
                        dispatch = None
                        pos = dest
                        continue
                flags.add("indirect")
                break
            dest = None if fma.search(target) else side.resolve(target, program)
            if dest is not None:
                flags.add("tail")
                pos = dest
                continue
            insns[-1] = "callq\t" + operands
            flags.add("opaque-tail")
            calls.append(target)
            flags.add("call")
            if fma.search(target):
                fma_calls += 1
            # The callee returns to whoever called the walked function.
            if not frames:
                break
            program, i, visited = frames.pop()
            pos = (program, i)
        elif call.match(mnemonic):
            dispatch = None
            target = call_target(operands, loaded)
            dest = None
            if not (fma.search(target) or len(frames) >= MAX_DEPTH):
                dest = side.resolve(target, program)
            if dest is not None:
                frames.append((program, i + 1, visited))
                visited = set()
                pos = dest
            else:
                insns.append(text)
                calls.append(target)
                flags.add("call")
                if fma.search(target):
                    fma_calls += 1
                pos = (program, i + 1)
        elif ret.match(mnemonic):
            dispatch = None
            if not frames:
                insns.append(text)
                break
            program, i, visited = frames.pop()
            pos = (program, i)
        elif mnemonic == "ud2":
            insns.append(text)
            flags.add("trap")
            break
        else:
            insns.append(text)
            pos = (program, i + 1)
    if len(insns) < SHORT_PATH:
        flags.add("short")
    encountered = {p.insns[fall - 1][0] for p, _, fall, _ in branches}
    if any(k not in encountered if isinstance(k, str) else k >= len(branches) for k in take):
        flags.add("take")
    return {"insns": insns, "calls": calls, "fma_calls": fma_calls,
            "flags": sorted(flags), "symbols": symbols, "branches": branches}


# --- The fast path, traced ----------------------------------------------------

# Runs inside gdb: breaks on every entry symbol, single-steps each hit until
# it returns to its caller, prints the executed addresses relative to the
# static ones (`STATIC`: symbol -> address in the file, so PIE relocation
# drops out).  `internal` breakpoints keep gdb's own numbering quiet.
GDB_TRACE = r"""
import gdb, json
gdb.execute("set pagination off")
gdb.execute("set confirm off")
gdb.execute("set width 0")
static = STATIC
for symbol in static:
    gdb.Breakpoint("*" + symbol, internal=True)
gdb.execute("run", to_string=True)


def reg(name):
    return int(gdb.parse_and_eval("$" + name))


traces = {}
while gdb.selected_inferior().pid:
    pc, sp = reg("pc"), reg("sp")
    ret = int(gdb.parse_and_eval("*(unsigned long *)$sp"))
    name = [s for s in static if int(gdb.parse_and_eval("(unsigned long)&" + s)) == pc]
    if not name:
        break
    base = pc - static[name[0]]
    pcs = []
    while len(pcs) < LIMIT:
        pcs.append(pc - base)
        gdb.execute("stepi", to_string=True)
        pc = reg("pc")
        if pc == ret and reg("sp") == sp + 8:
            break
    traces[name[0]] = pcs
    gdb.execute("continue", to_string=True)
print("TRACE " + json.dumps(traces))
"""
TRACE_LIMIT = 200000


def text_symbols(binary):
    """Sorted [(address, size, [names])] of the binary's sized text symbols."""
    rows = {}
    for line in run(["nm", "-S", "-n", "--defined-only", binary]).stdout.splitlines():
        parts = line.split()
        if len(parts) == 4 and parts[2] in "tTwW":
            rows.setdefault((int(parts[0], 16), int(parts[1], 16)), []).append(parts[3])
    return [(a, z, names) for (a, z), names in sorted(rows.items())]


def locate(table, pc):
    """The `text_symbols` row containing `pc`, or None."""
    i = bisect.bisect_right(table, (pc, float("inf"), [])) - 1
    if i >= 0 and pc < table[i][0] + table[i][1]:
        return table[i]
    return None


def trace(binary, name, static, argument):
    """{entry symbol: [static pc, ...]}: every instruction `analysis call`
    executes inside each entry (callees included, down to libm), in order."""
    script = GDB_TRACE.replace("STATIC", json.dumps(static)).replace("LIMIT", str(TRACE_LIMIT))
    with tempfile.NamedTemporaryFile("w", suffix=".py", delete=False) as file:
        file.write(script)
    try:
        out = run(["gdb", "-batch", "-nx", "-q", "-x", file.name, "--args", binary, "call", name]
                  + [str(a) for a in argument], env={"LD_BIND_NOW": "1"}, check=False)
    finally:
        os.unlink(file.name)
    match = re.search(r"^TRACE (.*)$", out.stdout, re.M)
    if not match:
        fail("gdb traced nothing for {} {}:\n{}".format(
            binary, argument, (out.stdout + out.stderr).strip()[-2000:]))
    traces = json.loads(match.group(1))
    for symbol in static:
        if symbol not in traces:
            fail("gdb never stopped in {} (`call` did not reach it)".format(symbol))
        if len(traces[symbol]) >= TRACE_LIMIT:
            fail("{} ran past {} instructions".format(symbol, TRACE_LIMIT))
    return traces


PADDING = re.compile(r"^(nop|xchg|int3|data16|cs)")
DISASM_HEAD = re.compile(r"^([0-9a-f]+) <(.*)>:$")
DISASM_INSN = re.compile(r"^\s*([0-9a-f]+):\s+(\S+)")


def disassemble(binary, names):
    """{start address: [(address, mnemonic)]} of every text symbol among `names`."""
    out = run(["llvm-objdump", "-d", "--no-show-raw-insn",
               "--disassemble-symbols=" + ",".join(sorted(names)), binary]).stdout
    functions, current = {}, None
    for line in out.splitlines():
        head = DISASM_HEAD.match(line)
        if head:
            current = functions.setdefault(int(head.group(1), 16), [])
            continue
        insn = DISASM_INSN.match(line)
        if insn and current is not None:
            current.append((int(insn.group(1), 16), insn.group(2)))
    return functions


def compatible(emitted, disassembled):
    """Recognize suffix and encoding aliases in emitted/disassembled instructions."""
    emitted, operands = emitted
    # LLVM uses REP BSF on pre-BMI targets when the input is nonzero. Its
    # encoding is also TZCNT, which objdump prints regardless of the target.
    if emitted == "rep" and operands.startswith("bsf"):
        emitted = "tzcnt" + operands.split(None, 1)[0][3:]
    # A local symbol's GOT load can be relaxed to its address by the linker.
    if emitted == "movq" and "@GOTPCREL(%rip)" in operands and disassembled == "leaq":
        return True
    return emitted == disassembled or emitted.startswith(disassembled) \
        or disassembled.startswith(emitted)


def align(program, start, listing):
    """{address: index into program.insns} pairing a function's disassembly
    with its emitted text from `start`; None when they are not the same code.
    The assembler's `.p2align` filler (`nopw`, `xchgw %ax, %ax`) has no line
    in the `.s` and is skipped."""
    mapping, i = {}, start
    for address, mnemonic in listing:
        if i < len(program.insns) and compatible(program.insns[i], mnemonic):
            mapping[address] = i
            i += 1
        elif not PADDING.match(mnemonic):
            return None
    return mapping


class Binary:
    """A built `analysis` executable: its text symbols, and the addresses of
    the functions a trace touched mapped onto one side's emitted assembly."""

    def __init__(self, path):
        self.path = path
        self.table = text_symbols(path)
        self.address = {n: a for a, _, names in self.table for n in names}
        self.cache = {}
        self.lock = threading.Lock()

    def mapping(self, side, start, names):
        """(address -> (program, index), address -> next address) for the
        function at `start`, or None when no program of `side` defines it."""
        key = (side, start)
        with self.lock:
            if key in self.cache:
                return self.cache[key]
        candidates = [(p, n) for n in names for p in side.programs if n in p.labels]
        result = None
        if candidates:
            listing = disassemble(self.path, {n for _, n in candidates}).get(start)
            if not listing:
                fail("llvm-objdump found no {} at {:#x} in {}".format(names, start, self.path))
            for program, name in candidates:
                mapped = align(program, program.labels[name], listing)
                if mapped is not None:
                    following = {a: b for (a, _), (b, _) in zip(listing, listing[1:])}
                    result = ({a: (program, i) for a, i in mapped.items()}, following)
                    break
            if result is None:
                fail("{} in {} is not the code {} emitted for it (different flags?)".format(
                    "/".join(names), self.path, "/".join(p.name for p, _ in candidates)))
        with self.lock:
            self.cache[key] = result
        return result


def traced_path(side, binary, pcs, leg=None):
    """The fast path from a trace: the same dict `walk` returns, holding what
    the traced call executed inside `side`'s assembly.  A call into this
    assembly is inlined (its `call` and `ret` dropped, its body kept) — except
    the accurate leg (`leg`, a predicate on targets, flags `leg-on-path`) and
    FMA thunks, which stay calls whose bodies are skipped, as is a call that
    leaves the assembly (libm, compiler-builtins: `call`). Opaque tail jumps
    become `callq` for llvm-mca's flat call cost (`opaque-tail`). A loop's every
    iteration is kept (`loop`)."""
    cfg = ISA["x86_64"]
    cond, jmp, ret, call, fma = (re.compile(cfg[k]) for k in ("cond", "jmp", "ret", "call", "fma"))
    located, following = {}, {}
    for pc in pcs:
        row = locate(binary.table, pc)
        if row is not None and row[0] not in located:
            located[row[0]] = binary.mapping(side, row[0], row[2])
    where = {}
    for mapped in located.values():
        if mapped is not None:
            where.update(mapped[0])
            following.update(mapped[1])
    insns, calls, flags, symbols, branches = [], [], set(), set(), []
    fma_calls, seen, resume, loaded = 0, set(), None, {}
    for n, pc in enumerate(pcs):
        if resume is not None:
            if pc != resume:
                continue
            resume = None
        if pc not in where:
            continue
        program, i = where[pc]
        after = pcs[n + 1] if n + 1 < len(pcs) else None
        if (id(program), i) in seen:
            flags.add("loop")
        seen.add((id(program), i))
        mnemonic, operands = program.insns[i]
        text = mnemonic + ("\t" + operands if operands else "")
        for symbol in MEMREF.findall(operands):
            symbols.add((program, symbol.split("@")[0]))
        got = GOT_LOAD.match(text)
        if got:
            loaded[got.group(2)] = got.group(1)
        elif operands.endswith(tuple(loaded)) and "," in operands:
            loaded.pop(operands.rsplit(",", 1)[1].strip(), None)
        if cond.match(mnemonic):
            insns.append(text)
            branches.append((program, operands, i + 1, after != following.get(pc)))
        elif call.match(mnemonic) or (jmp.match(mnemonic) and not operands.startswith(".L")):
            target = call_target(operands, loaded)
            is_leg = bool(leg and leg(target))
            if is_leg:
                flags.add("leg-on-path")
            if after in where and not (is_leg or fma.search(target)):
                if jmp.match(mnemonic):
                    insns.append(text)
                    flags.add("tail")
                continue
            insns.append("callq\t" + operands if jmp.match(mnemonic) else text)
            if jmp.match(mnemonic):
                flags.add("opaque-tail")
            calls.append(target)
            flags.add("call")
            if fma.search(target):
                fma_calls += 1
            # Skip the callee: back at the return address, or never (a tail
            # call returns to the traced function's caller).
            resume = following.get(pc) if call.match(mnemonic) else -1
        elif ret.match(mnemonic):
            if n == len(pcs) - 1:
                insns.append(text)
        elif mnemonic == "ud2":
            insns.append(text)
            flags.add("trap")
        else:
            insns.append(text)
    if len(insns) < SHORT_PATH:
        flags.add("short")
    return {"insns": insns, "calls": calls, "fma_calls": fma_calls,
            "flags": sorted(flags), "symbols": symbols, "branches": branches}


def header(path):
    """The first line of a walked `.s`: `# insns=N calls=N fma_calls=N flags=a,b|none [input=x,y,z]`."""
    line = "# insns={} calls={} fma_calls={} flags={}".format(
        len(path["insns"]), len(path["calls"]), path["fma_calls"],
        ",".join(path["flags"]) or "none")
    if "input" in path:
        line += " input=" + ",".join(str(a) for a in path["input"])
    return line


def write_path(file, path):
    file.parent.mkdir(parents=True, exist_ok=True)
    body = "".join("\t" + insn + "\n" for insn in path["insns"])
    file.write_text(header(path) + "\n" + body)


def leg_start(side, fast, leg):
    """(program, index) where the accurate leg begins, per the FN `leg` field."""
    if leg is None:
        return None
    if leg.startswith("sym:"):
        found = side.find(symbol_matcher(leg))
        if len(found) != 1:
            fail("leg {} matches {} labels: {}".format(
                leg, len(found), ", ".join(l for _, l in found) or "none"))
        program, label = found[0]
        return program, program.labels[label]
    if leg.startswith("branch:"):
        parts = leg.split(":")
        k = int(parts[-1])
        branches = fast["branches"]
        # `branch:je:-1` keeps a final Ziv gate distinct from a later sign
        # branch that AVX-512 folds away, changing the raw branch ordinal.
        if len(parts) == 3:
            branches = [b for b in branches if b[0].insns[b[2] - 1][0] == parts[1]]
        if not -len(branches) <= k < len(branches):
            fail("leg {} but the fast path met {} conditional branches".format(k, len(branches)))
        program, target, fallthrough, taken = branches[k]
        if taken:   # the fast path went that way: the leg is the other side
            return program, fallthrough
        dest = side.resolve(target, program)
        if dest is None:
            fail("branch target {} of leg {} is not defined".format(target, leg))
        return dest
    fail("bad leg spec " + leg)


def table_bytes(side, paths):
    """Bytes of every data symbol the paths reference, each distinct content
    counted once.  A `.L` label resolves in the program that referenced it, a
    global symbol across the side (`Side.data_of`); byte-identical data behind
    different labels is one entry, because rustc emits a `const` table once
    per codegen unit — the fast path inlined into the example reads one copy
    of `EXP2_T0`, the lib's accurate leg another, both `.Lanon.*` — and equal
    `.LCPI*` constants recur across the two files (those the linker merges;
    the tables it keeps, so the linked binary is larger than this count)."""
    total, seen = 0, set()
    for path in paths:
        for program, symbol in path["symbols"]:
            size, content = side.data_of(program, symbol)
            if size == 0 or content in seen:
                continue
            seen.add(content)
            total += size
    return total


def analyse(side, binary, pcs, argument, fn_side, want_acc):
    """One side of one function from its trace; return (fast path, acc path or None, summary)."""
    leg = fn_side["leg"] if fn_side else None
    fast = traced_path(side, binary, pcs,
                       leg=symbol_matcher(leg) if leg and leg.startswith("sym:") else None)
    fast["input"] = list(argument)
    acc = None
    if want_acc and leg:
        start = leg_start(side, fast, leg)
        acc = walk(side, start[0], start[1], take=fn_side["acc_take"])
    summary = {"insns": len(fast["insns"]), "calls": len(fast["calls"]),
               "fma_calls": fast["fma_calls"], "flags": fast["flags"],
               "input": list(argument),
               "branches": [[target, taken] for _, target, _, taken in fast["branches"]],
               "table_bytes": table_bytes(side, [fast] + ([acc] if acc else [])),
               "acc": None if acc is None else {
                   "insns": len(acc["insns"]), "calls": len(acc["calls"]),
                   "fma_calls": acc["fma_calls"], "flags": acc["flags"]}}
    return fast, acc, summary


# The crate id v0 mangling stamps on every `metallic` symbol (`Cs<hash>_8metallic`):
# cargo derives it from the exact rustc invocation, so a lib built separately
# from the example carries a different one and none of the example's calls
# resolve into it.
CRATE_ID = re.compile(r"Cs[A-Za-z0-9]+_8metallic")
# What rustc stamps at the end of every `.s` it emits.
IDENT = re.compile(r'^\s*\.ident\s+"rustc version (.*)"', re.M)


def hashed_twin(uplifted):
    """`examples/analysis-<hash>` behind the uplifted `examples/analysis`: cargo
    hardlinks the two (or copies when it cannot), so the twin shares the inode
    or, failing that, the bytes.  The hashed name is what `--emit=asm` stamps
    on the `.s` beside it."""
    twins = [f for f in uplifted.parent.glob(uplifted.name + "-*") if "." not in f.name]
    same = [f for f in twins
            if os.path.samefile(f, uplifted) or filecmp.cmp(f, uplifted, shallow=False)]
    if len(same) != 1:
        fail("{} hashed twins of {}: {}".format(
            len(same), uplifted, ", ".join(f.name for f in same) or "none"))
    return same[0].with_name(same[0].name + ".s")


def built_asm(artifacts, f128):
    """`([lib .s, example .s], executable)` of the build `artifacts` describe — never the
    newest file: a stable `--no-f128` build shares the target dir, and a
    nightly build after it is Fresh and re-emits nothing, so the newest `.s`
    can be the other toolchain's.  The lib's `.s` sits beside its rlib, the
    example's beside the hashed binary its uplifted copy links to; each must
    have been emitted by the toolchain invoked, and the lib must carry the
    crate id the example's symbols name (any file under legacy mangling)."""
    lib = example = executable = None
    for message in artifacts:
        target = message["target"]
        if target["name"] == "metallic" and "lib" in target["kind"]:
            rlibs = [Path(f) for f in message["filenames"] if f.endswith(".rlib")]
            if len(rlibs) != 1:
                fail("expected one metallic rlib, got: " + ", ".join(message["filenames"]))
            lib = rlibs[0].with_name(rlibs[0].name[len("lib"):-len(".rlib")] + ".s")
        elif target["name"] == "analysis" and "example" in target["kind"]:
            executable = Path(message["executable"])
            example = hashed_twin(executable)
    if lib is None or example is None:
        fail("cargo reported no " + ("metallic lib" if lib is None else "analysis example")
             + " artifact")
    version = rustc_version(f128).split(" ", 1)[1]
    for file in (lib, example):
        if not file.exists():
            fail("{} was not emitted (is --emit=asm in RUSTFLAGS?)".format(file))
        ident = IDENT.search(file.read_text())
        if not ident or ident.group(1) != version:
            fail("{} was emitted by rustc {}, not the {} invoked".format(
                file, ident.group(1) if ident else "?", version))
    ids = set(CRATE_ID.findall(example.read_text()))
    if len(ids) > 1:
        fail("{} names several metallic crate ids: {}".format(example, ", ".join(sorted(ids))))
    if ids and ids.pop() not in lib.read_text():
        fail("{} does not carry the crate id {} names".format(lib, example))
    return [lib, example], executable


def cmd_asm(args):
    levels = args.isa.split(",") if args.isa else list(ISA["x86_64"]["levels"])
    unknown = set(levels) - set(ISA["x86_64"]["levels"])
    if unknown:
        fail("unknown ISA levels: " + ", ".join(sorted(unknown)))
    names = selected(args.only, not args.no_f128)
    sys_dir = core_math_sys()
    for level in levels:
        cpu, march, _ = ISA["x86_64"]["levels"][level]
        model = None if level == ISA["x86_64"]["fma_count"] else mca_model(level)
        target_dir = ROOT / "target" / "analysis" / level
        # One build emits the lib's and the example's `.s` from the very units
        # the example links (`--emit` unions with cargo's own; `--target` keeps
        # the flags off build scripts).
        env = {"RUSTFLAGS": "-Ctarget-cpu={} --emit=asm -Ccodegen-units=1".format(cpu),
               "TARGET_CPU": cpu}
        artifacts = cargo(["build", "--release", "--example", "analysis"], target_dir, env,
                          not args.no_f128, artifacts=True)
        files, executable = built_asm(artifacts, not args.no_f128)
        if VERBOSE:
            print("+ walking " + " and ".join(map(str, files)), file=sys.stderr)
        metallic = Side.load(files)
        binary = Binary(executable)
        c_dir = target_dir / "core-math"
        c_dir.mkdir(parents=True, exist_ok=True)
        want_acc = level != ISA["x86_64"]["fma_count"]

        def compile_c(name, prec):
            source = c_source(sys_dir, name, prec)
            out = c_dir / (name + ".s")
            cmd = ["clang", "-O3", "-fPIC", "-ffunction-sections", "-fdata-sections",
                   "-march=" + march, "-w", "-S", "-I", source.parent]
            if prec == "f128":
                cmd += ["-include", sys_dir / "lib" / "f128-decls.h"]
            run(cmd + ["-o", out, source])
            return out

        def trace_fn(name):
            entries = ["metallic_" + name] + (["cr_" + name] if name not in NO_CORE_MATH else [])
            missing = [e for e in entries if e not in binary.address]
            if missing:
                fail("{} has no {} symbol".format(executable, "/".join(missing)))
            return trace(executable, name, {e: binary.address[e] for e in entries}, input_of(name))

        with ThreadPoolExecutor(max_workers=args.jobs) as pool:
            c_files = {name: pool.submit(compile_c, name, prec)
                       for name, prec in names if name not in NO_CORE_MATH}
            traces = {name: pool.submit(trace_fn, name) for name, _ in names}
        for name, prec in names:
            sides = [("metallic", metallic, "metallic_" + name, side_of(name, "m"))]
            if name in c_files:
                sides.append(("core-math", Side.load([c_files[name].result()]),
                              "cr_" + name, side_of(name, "c")))
            for side_name, side, label, fn_side in sides:
                fast, acc, summary = analyse(side, binary, traces[name].result()[label],
                                             input_of(name), fn_side, want_acc)
                summary["mcpu"] = model
                out = ROOT / "analysis" / level / side_name
                write_path(out / (name + ".s"), fast)
                if acc is not None:
                    write_path(out / (name + ".acc.s"), acc)
                # The paths just changed, so llvm-mca output priced from the
                # old ones is stale: drop the `.txt`s (and an `.acc.s` no
                # longer walked) until `mca` prices the new bodies — the JSON
                # carries no `cycles` until then and `render` shows `—`.
                for stale in ([".acc.s"] if acc is None else []) + [".txt", ".acc.txt"]:
                    (out / (name + stale)).unlink(missing_ok=True)
                write_json(out / (name + ".json"), summary)
                print("{:6} {:9} {:10} {}".format(level, side_name, name, header(fast)[2:]))
    if args.command != "all" and any(l != ISA["x86_64"]["fma_count"] for l in levels):
        print("analysis: walked paths rewritten; run `mca` to price them again", file=sys.stderr)


def write_json(file, data):
    file.parent.mkdir(parents=True, exist_ok=True)
    temporary = file.with_suffix(file.suffix + ".tmp")
    temporary.write_text(json.dumps(data, indent=1, sort_keys=True, ensure_ascii=False) + "\n")
    temporary.replace(file)


def read_json(file):
    return json.loads(Path(file).read_text())


# --- llvm-mca ---------------------------------------------------------------

def mca_body(file):
    """Instructions of a walked `.s` without its header line."""
    lines = Path(file).read_text().splitlines()
    if lines and lines[0].startswith("#"):
        lines = lines[1:]
    return "\n".join(lines) + "\n"


def mca(body, model):
    """(cycles per iteration, llvm-mca stdout) for an instruction body; (None, "") when empty."""
    if not body.strip():
        return None, ""
    out = run(["llvm-mca", "-mcpu=" + model, "-iterations=" + str(ITERATIONS),
               "-instruction-info=false", "-resource-pressure=false"], stdin=body)
    match = re.search(r"^Total Cycles:\s+(\d+)", out.stdout, re.M)
    if not match:
        fail("llvm-mca printed no Total Cycles for model " + model)
    cycles = int(match.group(1)) / ITERATIONS
    return (int(cycles) if ITERATIONS == 1 else round(cycles, 1)), out.stdout


def cmd_mca(args):
    names = {n for n, _ in selected(args.only, not args.no_f128)}
    models = args.mcpu.split(",") if args.mcpu else None
    for level in ISA["x86_64"]["cycles"]:
        for side_name in ("metallic", "core-math"):
            source = ROOT / "analysis" / level / side_name
            if not source.is_dir():
                continue
            for asm_file in sorted(source.glob("*.s")):
                name = asm_file.name[:-2]
                if name.endswith(".acc") or name not in names:
                    continue
                json_file = source / (name + ".json")
                for model in models or [mca_model(level)]:
                    out_dir = source if models is None else \
                        ROOT / "analysis" / "mcpu" / model / level / side_name
                    summary = read_json(json_file) if models is None and json_file.exists() else {}
                    summary["mcpu"] = model
                    for suffix, key in ((".s", "cycles"), (".acc.s", "acc_cycles")):
                        if not (source / (name + suffix)).exists():
                            summary[key] = None
                            continue
                        cycles, text = mca(mca_body(source / (name + suffix)), model)
                        summary[key] = cycles
                        out_dir.mkdir(parents=True, exist_ok=True)
                        (out_dir / (name + suffix[:-2] + ".txt")).write_text(text)
                    write_json(out_dir / (name + ".json"), summary)
                    print("{:6} {:9} {:10} {:12} cycles={} acc_cycles={}".format(
                        level, side_name, name, model, summary["cycles"], summary["acc_cycles"]))


# --- Coverage ---------------------------------------------------------------

def probes(binary):
    """`analysis list` as (name, prec, arity, core_math) tuples, checked against FUNCTIONS."""
    rows = []
    # The instrumented binary writes a profile wherever it runs; keep it out of the tree.
    env = {"LLVM_PROFILE_FILE": str(Path(binary).parent / "list.profraw")}
    for line in run([binary, "list"], env=env).stdout.splitlines():
        name, prec, arity, core = line.split()
        rows.append((name, prec, int(arity), core == "1"))
    listed = {n for n, _, _, _ in rows}
    for prec, names in FUNCTIONS.items():
        missing = set(names) - listed
        if missing and (prec != "f128" or any(n.endswith("q") for n in listed)):
            fail("probes missing from `analysis list`: " + ", ".join(sorted(missing)))
    for name, prec, _, core in rows:
        if precision_of(name) != prec or core == (name in NO_CORE_MATH):
            fail("`analysis list` disagrees with FUNCTIONS/NO_CORE_MATH for " + name)
    return rows


def parse_lcov(text):
    """{path: {line or (line, block, edge): count}} from LLVM's LCOV export."""
    files, current, counts = {}, None, {}
    for line in text.splitlines():
        if line.startswith("SF:"):
            current, counts = line[3:], {}
            files[current] = counts
        elif line.startswith("DA:") and current is not None:
            number, count = line[3:].split(",")[:2]
            counts[int(number)] = counts.get(int(number), 0) + int(count)
        elif line.startswith("BRDA:") and current is not None:
            number, block, edge, count = line[5:].split(",")
            key = (int(number), int(block), int(edge))
            counts[key] = counts.get(key, 0) + (0 if count == "-" else int(count))
    return files


def anchor_count(coverage, anchor):
    """Execution count of one `cov` anchor; asserts the substring is unique in the file."""
    spec = {"source": anchor} if isinstance(anchor, str) else anchor
    suffix, needle = spec["source"].split(": ", 1)
    matches = [path for path in coverage if path.endswith(suffix)]
    if len(matches) != 1:
        fail("anchor file {} matches {} coverage records".format(suffix, len(matches)))
    path = matches[0]
    lines = [i + 1 for i, text in enumerate(Path(path).read_text().splitlines())
             if needle in text]
    if len(lines) != 1:
        fail("anchor {!r} matches {} lines in {}".format(needle, len(lines), path))
    line = lines[0] + spec.get("offset", 0)
    key = (line, *spec["branch"]) if "branch" in spec else line
    if key not in coverage[path]:
        fail("no coverage record for {} of {} ({!r})".format(key, path, needle))
    return coverage[path][key]


def leg_counts(coverage, name):
    """(metallic count, CORE-MATH count or None) for one function's anchors."""
    counts = []
    for key in ("m", "c"):
        side = side_of(name, key)
        counts.append(None if side is None else
                      sum(anchor_count(coverage, a) for a in side["cov"]))
    return counts[0], counts[1]


def cmd_legs(args):
    f128 = not args.no_f128
    # Validate the selection before the build (a `--no-f128` build into the
    # shared coverage dir also swaps the uplifted binary for the stable one).
    wanted = {n for n, _ in selected(args.only, f128)}
    cov = ROOT / "target" / "analysis" / "cov"
    cargo(["build", "--release", "--example", "analysis"], cov,
          {"RUSTFLAGS": "-Cinstrument-coverage -Ctarget-cpu=native",
           "CFLAGS": "-fprofile-instr-generate -fcoverage-mapping",
           "TARGET_CPU": "native"}, f128)
    binary = cov / TRIPLE / "release" / "examples" / "analysis"
    # A run re-sweeps the selected functions — everything, the `--only`
    # list, or all but the q functions under `--no-f128` — and merges into
    # the stored file: every other row keeps its record, and a bucket no
    # selected function falls in keeps its stored sample count (the Method
    # sentence quotes one count per bucket).  Under `--only` a bucket no
    # flag names inherits its stored count (the default without one) and a
    # flag that disagrees with it is refused rather than mixed into the
    # bucket's other rows; without `--only` every row of a touched bucket is
    # re-swept, so the flags, or the defaults, apply outright.
    file = ROOT / "analysis" / "legs.json"
    data = read_json(file) if file.exists() else {"fns": {}}
    stored = data.get("meta", {}).get("samples", {})
    inherit = stored if args.only else {}
    explicit = {"f32": args.f32_samples, "f32_multi": args.f64_samples,
                "f64": args.f64_samples, "f128": args.f128_samples}
    given = {b: explicit[b] if explicit[b] is not None
             else inherit[b] if inherit.get(b) is not None else DEFAULT_SAMPLES[b]
             for b in DEFAULT_SAMPLES}
    jobs, touched = [], set()
    for name, prec, arity, _ in probes(binary):
        if name not in wanted:
            continue
        if prec == "f32" and arity == 1:
            bucket, rank = "f32", 0
        elif prec == "f128":
            bucket, rank = "f128", 2
        else:
            bucket, rank = "f32_multi" if prec == "f32" else "f64", 1
        touched.add(bucket)
        jobs.append((rank, name, given[bucket]))
    jobs.sort()
    mixed = {b: (stored[b], given[b]) for b in sorted(touched)
             if stored.get(b) is not None and stored[b] != given[b]}
    if mixed and args.only:
        fail("--only would mix sample counts with analysis/legs.json ({}); drop the flag to "
             "inherit the stored count, or re-sweep without --only".format(", ".join(
                 "{}: stored {} vs {}".format(b, old, new) for b, (old, new) in mixed.items())))
    samples = {b: given[b] if b in touched else stored.get(b) for b in DEFAULT_SAMPLES}

    def profile(name, samples):
        profraw = cov / (name + ".profraw")
        profdata = cov / (name + ".profdata")
        out = run([binary, name, str(samples)], env={"LLVM_PROFILE_FILE": str(profraw)}).stdout
        match = re.search(r"^finite=(\d+)$", out, re.M)
        if not match:
            fail("no finite= line from the {} sweep".format(name))
        if int(match.group(1)) != (samples or 0xff000000):
            fail("unexpected finite-input denominator from the {} sweep".format(name))
        run(["llvm-profdata", "merge", "-sparse", "-o", profdata, profraw])
        lcov = run(["llvm-cov", "export", "-format=lcov", "-instr-profile=" + str(profdata),
                    binary]).stdout
        return int(match.group(1)), parse_lcov(lcov)

    def sweep(name, samples):
        finite, coverage = profile(name, samples)
        metallic, core = leg_counts(coverage, name)
        if name == "tanpi":
            # The tiny band reaches the database without either polynomial
            # gate; replay the same draws there to count that disjoint path.
            _, tiny = profile("tanpi-tiny", samples)
            core += anchor_count(tiny, "binary64/tanpi/tanpi.c: "
                                 "static __attribute__((noinline)) double as_tanpi_database(double x, double f){")
        return name, {"finite": finite, "metallic": metallic, "core_math": core}

    data["meta"] = {
        "commit": run(["git", "rev-parse", "HEAD"]).stdout.strip(),
        "date": date.today().isoformat(),
        "rustc": rustc_version(f128), "clang": clang_version(),
        "samples": samples}
    known = {n for names in FUNCTIONS.values() for n in names}
    data["fns"] = {n: r for n, r in data.get("fns", {}).items() if n in known and n not in wanted}
    # Keep completed rows if a long exhaustive sweep is interrupted.
    write_json(file, data)
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        futures = [pool.submit(sweep, name, samples) for _, name, samples in jobs]
        for future in as_completed(futures):
            name, record = future.result()
            if not record["finite"] or any(v is not None and not 0 <= v <= record["finite"]
                                           for k, v in record.items() if k != "finite"):
                fail("invalid accurate-leg counts for {}: {}".format(name, record))
            data["fns"][name] = record
            write_json(file, data)
            print("{:10} finite={} metallic={} core_math={}".format(
                name, record["finite"], record["metallic"], record["core_math"]))


# --- Calibration ------------------------------------------------------------

def cpu_model():
    for line in Path("/proc/cpuinfo").read_text().splitlines():
        if line.startswith("model name"):
            return line.split(":", 1)[1].strip()
    fail("no model name in /proc/cpuinfo")


def criterion_median(target_dir, lane, name):
    file = target_dir / "criterion" / (lane + "__" + name) / "new" / "estimates.json"
    if not file.exists():
        return None
    return round(read_json(file)["median"]["point_estimate"], 2)


def cmd_calibrate(args):
    # alloca (Criterion dependency) enables clang LTO, which GNU ld cannot read.
    env = {"RUSTFLAGS": "-Ctarget-cpu=native", "TARGET_CPU": "native", "CFLAGS": "-fno-lto"}
    # Isolate cc's native objects too: TARGET_CPU alone is not a cc cache key.
    target_dir = ROOT / "target" / "analysis" / "calibration"
    load = list(os.getloadavg())
    available = int(re.search(r"MemAvailable:\s+(\d+)", Path("/proc/meminfo").read_text())[1])
    cargo(["bench", "--bench", "expf", "--bench", "exp"], target_dir, env, False)
    if not args.no_f128:
        cargo(["bench", "--bench", "expq"], target_dir, env, True)
    record = {"date": date.today().isoformat(), "mca_native": native_model(),
              "load_before": load, "load_after": list(os.getloadavg()),
              "logical_cpus": os.cpu_count(), "mem_available_kib": available}
    for name in CALIBRATION:
        if name.endswith("q") and args.no_f128:
            continue
        record[name] = {"metallic": criterion_median(target_dir, "metallic", name),
                        "core_math": criterion_median(target_dir, "core_math", name)}
    file = ROOT / "analysis" / "calibration.json"
    data = read_json(file) if file.exists() else {}
    data[cpu_model()] = record
    write_json(file, dict(sorted(data.items())))


# --- Report -----------------------------------------------------------------

def load_side(level, side_name, name):
    file = ROOT / "analysis" / level / side_name / (name + ".json")
    return read_json(file) if file.exists() else None


def cycles_cell(summary, key, flags_key=None):
    """`64.1†‡` style cell for one side; `—` without data."""
    if summary is None or summary.get(key) is None:
        return "—"
    flags = summary["flags"] if flags_key is None else (summary.get(flags_key) or {}).get("flags", [])
    if set(flags) & {"indirect", "trap", "end", "take", "leg-on-path"}:
        return "incomplete"
    return number(summary[key]) + ("†" if "call" in flags else "") + \
        ("‡" if "short" in flags else "")


def pair(metallic, core, has_core):
    return metallic + " / " + (core if has_core else "—")


def percentage(record, key):
    if record is None or record.get(key) is None:
        return "?"
    count, finite = record[key], record["finite"]
    return "0" if count == 0 else "{:.3g}".format(100 * count / finite)


def number(value):
    if value is None:
        return "?"
    return str(value) if isinstance(value, int) else "{:g}".format(value)


def count(value):
    return "?" if value is None else "{:,}".format(value)


def escape(text):
    return str(text).replace("|", "\\|")


def function_row(name, legs):
    has_core = name not in NO_CORE_MATH
    cells = ["`" + name + "`"]
    for level in ISA["x86_64"]["cycles"]:
        m, c = load_side(level, "metallic", name), load_side(level, "core-math", name)
        cells.append(pair(cycles_cell(m, "cycles"), cycles_cell(c, "cycles"), has_core))
    v2 = ISA["x86_64"]["fma_count"]
    m, c = load_side(v2, "metallic", name), load_side(v2, "core-math", name)
    cells.append(pair(number(m and m.get("fma_calls")), number(c and c.get("fma_calls")), has_core))
    record = legs.get(name)
    cells.append(pair(percentage(record, "metallic"), percentage(record, "core_math"), has_core))
    m, c = load_side("v3", "metallic", name), load_side("v3", "core-math", name)
    cells.append(pair(cycles_cell(m, "acc_cycles", "acc"), cycles_cell(c, "acc_cycles", "acc"), has_core))
    sm, sc = side_of(name, "m"), side_of(name, "c")
    cells.append(pair(escape(sm["prec"]).replace(" / ", " or "),
                      escape(sc["prec"]).replace(" / ", " or ") if sc else "—", has_core))
    cells.append(pair(number(m and m.get("table_bytes")), number(c and c.get("table_bytes")), has_core))
    return "| " + " | ".join(cells) + " |"


def function_table(prec, legs, native_label):
    lines = ["| Function | v3 | v4 | native ({}) | FMA calls @v2 | Accurate leg % | "
             "Accurate cycles (v3) | Precision | Table bytes |".format(native_label),
             "| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: |"]
    lines += [function_row(name, legs) for name in FUNCTIONS[prec]]
    return "\n".join(lines)


def calibration_table(calibration):
    lines = ["| Host | Function | metallic ns | CORE-MATH ns | metallic cycles (native) | "
             "CORE-MATH cycles (native) |", "| --- | --- | ---: | ---: | ---: | ---: |"]
    for host, record in sorted(calibration.items()):
        for name in CALIBRATION:
            lane = record.get(name)
            if lane is None:
                continue
            m, c = load_side("native", "metallic", name), load_side("native", "core-math", name)
            lines.append("| {} ({}, {}) | `{}` | {} | {} | {} | {} |".format(
                escape(host), record.get("date", "?"), record.get("mca_native", "?"), name,
                number(lane.get("metallic")), number(lane.get("core_math")),
                cycles_cell(m, "cycles"), cycles_cell(c, "cycles")))
    for host, record in sorted(calibration.items()):
        if "load_before" in record:
            lines += ["", "{}: {} logical CPUs; 1-minute load {:.2f} before / {:.2f} after; "
                      "{:.1f} GiB available memory before. Timings under load are context, "
                      "not an idle-host calibration.".format(
                          escape(host), record["logical_cpus"], record["load_before"][0],
                          record["load_after"][0], record["mem_available_kib"] / 2**20)]
    return "\n".join(lines)


def native_label():
    for side_name in ("metallic", "core-math"):
        directory = ROOT / "analysis" / "native" / side_name
        for file in sorted(directory.glob("*.json")) if directory.is_dir() else []:
            model = read_json(file).get("mcpu")
            if model:
                return model
    return "native"


def samples_text(meta):
    """The swept buckets' denominators; a bucket recorded as null was not swept."""
    samples = meta.get("samples", {})
    parts = []
    f32 = samples.get("f32")
    if f32 is not None:
        parts.append("f32 univariate " + ("exhaustive (every finite bit pattern)" if f32 == 0
                                          else count(f32) + " samples"))
    drawn = ["{} {}".format(label, count(samples[key]))
             for key, label in (("f32_multi", "f32 bivariate and trivariate"), ("f64", "f64"),
                                ("f128", "f128")) if samples.get(key) is not None]
    if drawn:
        parts.append(" and ".join(filter(None, [", ".join(drawn[:-1]), drawn[-1]]))
                     + " representation-uniform finite samples")
    return "; ".join(parts) or "no sweep recorded"


def cmd_render(args):
    legs_file = ROOT / "analysis" / "legs.json"
    legs = read_json(legs_file) if legs_file.exists() else {"meta": {}, "fns": {}}
    calibration_file = ROOT / "analysis" / "calibration.json"
    calibration = read_json(calibration_file) if calibration_file.exists() else {}
    label = native_label()
    meta = legs.get("meta", {})
    sections = [
        "# Static analysis",
        "Static, per-function cost estimates for every public function, metallic beside "
        "CORE-MATH: the llvm-mca latency of the fast path at three ISA levels, the number of "
        "FMA calls at x86-64-v2, the exact or sampled fraction of finite inputs that reach the "
        "accurate leg, that leg's own static cycles, the working precision of each leg, and "
        "the bytes of tables the two paths touch. "
        "The cycle estimates depend on the emitted code, a fixed representative input and "
        "LLVM's scheduling models; the accurate-leg fractions are deterministic counts. "
        "Neither depends on runner timing or machine load. The separate calibration rows "
        "are measurements and do depend on the host's conditions. "
        "What the estimate cannot see is branch prediction, cache behaviour beyond an L1 hit, "
        "and the random-input generation a benchmark loop pays alongside the function.",
        "Regenerate with `python3 tools/analysis.py all` "
        "(artefacts under `analysis/`; commit `{}`, {}).".format(
            meta.get("commit", "?")[:12], meta.get("date", "?")),
        "Coverage toolchain: `{}`; `{}`. Use LLVM coverage tools from the same major "
        "version as rustc and clang.".format(meta.get("rustc", "?"), meta.get("clang", "?")),
        "## Method",
        "**Function.** One row per public function, cells `metallic / CORE-MATH`; `—` where "
        "the `core_math` crate has no binding (`fma*`, `frexp*`, `ldexp*`, `round*`, "
        "`compound`, and `cosq sinq tanq log2q log10q log1pq powq`).",
        "**v3, v4, native.** Cycles of the fast path from `llvm-mca -mcpu=x86-64-v3`, "
        "`x86-64-v4` and the host model (`{}`), each on the assembly rustc and clang emit for "
        "that level. GDB traces one call from the wrapper's entry (`metallic_<fn>`, "
        "`cr_<fn>`) to its return. Both sides receive the same input: `(x,y,z) = "
        "(1.7,0.7,0.3)`, with `x=0.7` for asin/acos/atanh, asinpi/acospi and erf/erfc, "
        "`x=4.5` for gamma, and `y=3` for ldexp; suffix variants share these values. "
        "Every executed branch and loop iteration is retained. Calls defined in the emitted "
        "assembly are expanded as if inlined; FMA thunks and external calls remain opaque. "
        "Opaque tail jumps are represented as calls for llvm-mca (`opaque-tail` in the "
        "path's flags). "
        "The input and branch decisions are recorded beside each path under `analysis/`. "
        "Directives, labels and comments are stripped before llvm-mca. Generation requires "
        "GNU/Linux x86-64, GDB, nm, llvm-objdump, and a host able to execute the selected "
        "ISA levels; `--mcpu` can price the recorded code for other scheduling models. "
        "`-iterations={}` runs the block once on an empty pipeline, so the figure is the "
        "latency of one call from its first dispatch to its last retirement — not "
        "throughput, and not the steady state of a dependent loop (the Zen 3/4 models drop "
        "loop-carried dependencies through eliminated moves, see Caveats). `†` marks a "
        "path with a call on it (llvm-mca prices a call at a flat 100 cycles); `‡` a path "
        "shorter than {} instructions, including trivial functions and exact-case exits. "
        "This is one path's estimate, not a full-domain average.".format(label, ITERATIONS, SHORT_PATH),
        "**FMA calls @v2.** x86-64-v2 has no FMA instruction. Exact fused multiply-adds use "
        "calls — metallic's runtime-dispatched `force_fma`, CORE-MATH's `fma@PLT`; the "
        "column counts those calls instead of pricing them. Metallic's `fast_mul_add` "
        "uses separate multiply/add instructions at v2 and does not contribute a call.",
        "**Accurate leg %.** The share of finite inputs whose evaluation entered the accurate "
        "leg, counted with LLVM source coverage (`-C instrument-coverage`, "
        "`-fprofile-instr-generate`) at the leg's definition line or the first statement of "
        "its inline fallback block; branch counts handle early-return lookup tables. "
        "CORE-MATH tanpi's tiny band replays the same draws separately, because its shared "
        "database gate merges with the polynomial fallbacks in aggregate coverage. "
        "Denominator: finite inputs, including those outside "
        "the function's domain — {}. `ldexp*` draws its integer exponent in "
        "`-2200..=2200`, matching its benchmark band. `0` means no accurate-leg entries observed (exact "
        "only for an exhaustive sweep or an implementation with no accurate leg); other "
        "values are three significant digits of the percentage.".format(
            samples_text(meta)),
        "**Accurate cycles (v3).** llvm-mca cycles of one static path from the accurate "
        "leg's entry. Conditional branches fall through except explicit `acc_take` entries "
        "in `FN`; loops count once and local calls expand. This omits repeated iterations "
        "and other branches; jump tables choose their first arm. `incomplete` marks a "
        "path the walker could not finish. Opaque calls use llvm-mca's flat cost, so the estimate is "
        "neither a measured cost nor a guaranteed lower bound.",
        "**Precision.** The working precision of the fast leg and of the accurate leg, "
        "hand-maintained in `tools/analysis.py` (`FN`). Arrows separate successive tiers; "
        "`or` denotes alternative bands. `?` means unrecorded.",
        "**Table bytes.** Every read-only symbol a memory operand on either path references "
        "(`sym(%rip)`, `sym(,%reg,8)`, `.LCPI*`, `.Lanon*`), sized from the data directives "
        "after its label in the full assembly and summed once per matching set of emitted "
        "data directives: rustc "
        "emits a `const` table once per codegen unit, so the copy the inlined fast path reads "
        "and the copy the library's accurate leg reads are byte-identical and count once, as "
        "do equal constants behind different local labels. The result is the constant "
        "footprint the two paths can touch, whether or not a given input reads all of it.",
        "## Calibration",
        "Criterion medians of the corresponding functions, beside the representative native "
        "path's llvm-mca estimate. Criterion draws from each benchmark's band, which can "
        "execute other paths, and includes the random draw; the estimate does not.",
        calibration_table(calibration) if calibration else "_No calibration recorded._",
        "## Binary32",
        function_table("f32", legs.get("fns", {}), label),
        "## Binary64",
        function_table("f64", legs.get("fns", {}), label),
        "## Binary128",
        function_table("f128", legs.get("fns", {}), label),
        "## Caveats",
        "- llvm-mca's AMD models have historically been the weaker ones; the Zen model here is "
        "the closest to the host, not a measurement of it, and the calibration rows above are "
        "the only check on its scale. Its znver3/znver4 register-file model also loses a "
        "loop-carried dependency when a zero idiom hits the source of an eliminated move, "
        "which is why the columns are single-pass latencies rather than the steady state of "
        "a dependent loop: the single pass includes the front-end fill of the block, a few "
        "cycles for a short path and more for a long one.\n"
        "- A `call` on a path is priced at 100 cycles by llvm-mca whatever the callee costs; "
        "rows marked `†` are estimates of the wrapper, not of the callee.\n"
        "- The accurate-leg fraction is a property of the input distribution: "
        "representation-uniform finite inputs, exhaustive for f32 univariate functions when "
        "`--f32-samples=0`. A "
        "workload concentrated near a function's hard cases pays the accurate leg far more "
        "often.\n"
        "- Table loads are assumed to hit L1; the table-bytes column says how much must be "
        "resident for that to hold. It counts byte-identical data once, while the linked "
        "binary holds rustc's per-codegen-unit copies of a `const` table separately (the "
        "example's inlined fast path and the library's accurate leg each read their own), so "
        "a crate that inlines the fast path carries more than the column says.\n"
        "- Criterion rows include the RNG draw of each input and the loop around the call.\n"
        "- The representative input selects one path. Magnitude, sign, argument reduction "
        "and exact cases can select very different paths; use the existing Criterion "
        "benches for a workload comparison. Internal call/return overhead is omitted when "
        "the callee is expanded into the path.",
    ]
    (ROOT / "ANALYSIS.md").write_text("\n\n".join(sections) + "\n")
    print("wrote " + str(ROOT / "ANALYSIS.md"))


# --- CLI --------------------------------------------------------------------

def cmd_all(args):
    for command in (cmd_asm, cmd_mca, cmd_legs, cmd_calibrate, cmd_render):
        command(args)


def global_options(parser, top):
    """`--only`, `--no-f128`, `-v` on the top-level parser and, so they are accepted
    after the subcommand too, on every subparser — with suppressed defaults there,
    since a subparser's defaults would otherwise overwrite the top-level values."""
    suppress = {} if top else {"default": argparse.SUPPRESS}
    parser.add_argument("--only", help="comma-separated function names", **suppress)
    parser.add_argument("--no-f128", action="store_true",
                        help="drop the binary128 functions and build on stable", **suppress)
    parser.add_argument("-v", "--verbose", action="store_true", help="echo commands", **suppress)


def main():
    global ROOT, VERBOSE
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    global_options(parser, top=True)
    parser.add_argument("--root", help=argparse.SUPPRESS)
    sub = parser.add_subparsers(dest="command", required=True)
    asm = sub.add_parser("asm", help="emit and walk the assembly per ISA level")
    asm.add_argument("--isa", help="comma-separated levels (default: all)")
    asm.add_argument("--jobs", type=int, default=8, help="parallel clang invocations")
    asm.set_defaults(func=cmd_asm)
    mca_p = sub.add_parser("mca", help="price the walked paths with llvm-mca")
    mca_p.add_argument("--mcpu", help="ad-hoc comma-separated models (written under analysis/mcpu/)")
    mca_p.set_defaults(func=cmd_mca)
    legs = sub.add_parser("legs", help="count accurate-leg entries with source coverage")
    legs.add_argument("--jobs", type=int, default=8)
    # No argparse default: under --only an unset flag inherits the count
    # stored in analysis/legs.json; DEFAULT_SAMPLES applies otherwise.
    legs.add_argument("--f32-samples", type=int,
                      help="f32 univariate probes; 0 = exhaustive (default)")
    legs.add_argument("--f64-samples", type=int,
                      help="f64 and f32 bivariate/trivariate probes (default 2^30)")
    legs.add_argument("--f128-samples", type=int, help="f128 probes (default 2^28)")
    legs.set_defaults(func=cmd_legs)
    calibrate = sub.add_parser("calibrate", help="Criterion medians for expf, exp, expq")
    calibrate.set_defaults(func=cmd_calibrate)
    render = sub.add_parser("render", help="write ANALYSIS.md from analysis/")
    render.set_defaults(func=cmd_render)
    everything = sub.add_parser("all", help="asm, mca, legs, calibrate, render")
    everything.add_argument("--isa")
    everything.add_argument("--jobs", type=int, default=8)
    for name in ("--f32-samples", "--f64-samples", "--f128-samples"):
        everything.add_argument(name, type=int)
    everything.set_defaults(func=cmd_all, mcpu=None)
    for subparser in (asm, mca_p, legs, everything, calibrate, render):
        global_options(subparser, top=False)
    args = parser.parse_args()
    VERBOSE = args.verbose
    if args.root:
        ROOT = Path(args.root).resolve()
    args.only = [n for n in args.only.split(",") if n] if args.only else None
    args.func(args)


if __name__ == "__main__":
    main()
