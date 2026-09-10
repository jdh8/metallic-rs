# Static analysis

Static, per-function cost estimates for every public function, metallic beside CORE-MATH: the llvm-mca latency of the fast path at three ISA levels, the number of FMA calls at x86-64-v2, the exact or sampled fraction of finite inputs that reach the accurate leg, that leg's own static cycles, the working precision of each leg, and the bytes of tables the two paths touch. The cycle estimates depend on the emitted code, a fixed representative input and LLVM's scheduling models; the accurate-leg fractions are deterministic counts. Neither depends on runner timing or machine load. The separate calibration rows are measurements and do depend on the host's conditions. What the estimate cannot see is branch prediction, cache behaviour beyond an L1 hit, and the random-input generation a benchmark loop pays alongside the function.

Regenerate with `python3 tools/analysis.py all` (artefacts under `analysis/`; commit `aa21a92b02a9`, 2026-09-10).

Coverage toolchain: `rustc 1.98.0-nightly (c1b22f44c 2026-06-17)`; `clang version 22.1.8 (Fedora 22.1.8-4.fc44)`. Use LLVM coverage tools from the same major version as rustc and clang.

## Method

**Function.** One row per public function, cells `metallic / CORE-MATH`; `—` where the `core_math` crate has no binding (`fma*`, `frexp*`, `ldexp*`, `round*`, `compound`, and `cosq sinq tanq log2q log10q log1pq powq`).

**v3, v4, native.** Cycles of the fast path from `llvm-mca -mcpu=x86-64-v3`, `x86-64-v4` and the host model (`znver4`), each on the assembly rustc and clang emit for that level. GDB traces one call from the wrapper's entry (`metallic_<fn>`, `cr_<fn>`) to its return. Both sides receive the same input: `(x,y,z) = (1.7,0.7,0.3)`, with `x=0.7` for asin/acos/atanh, asinpi/acospi and erf/erfc, `x=4.5` for gamma, and `y=3` for ldexp; suffix variants share these values. Every executed branch and loop iteration is retained. Calls defined in the emitted assembly are expanded as if inlined; FMA thunks and external calls remain opaque. Opaque tail jumps are represented as calls for llvm-mca (`opaque-tail` in the path's flags). The input and branch decisions are recorded beside each path under `analysis/`. Directives, labels and comments are stripped before llvm-mca. Generation requires GNU/Linux x86-64, GDB, nm, llvm-objdump, and a host able to execute the selected ISA levels; `--mcpu` can price the recorded code for other scheduling models. `-iterations=1` runs the block once on an empty pipeline, so the figure is the latency of one call from its first dispatch to its last retirement — not throughput, and not the steady state of a dependent loop (the Zen 3/4 models drop loop-carried dependencies through eliminated moves, see Caveats). `†` marks a path with a call on it (llvm-mca prices a call at a flat 100 cycles); `‡` a path shorter than 20 instructions, including trivial functions and exact-case exits. This is one path's estimate, not a full-domain average.

**FMA calls @v2.** x86-64-v2 has no FMA instruction. Exact fused multiply-adds use calls — metallic's runtime-dispatched `force_fma`, CORE-MATH's `fma@PLT`; the column counts those calls instead of pricing them. Metallic's `fast_mul_add` uses separate multiply/add instructions at v2 and does not contribute a call.

**Accurate leg %.** The share of finite inputs whose evaluation entered the accurate leg, counted with LLVM source coverage (`-C instrument-coverage`, `-fprofile-instr-generate`) at the leg's definition line or the first statement of its inline fallback block; branch counts handle early-return lookup tables. CORE-MATH tanpi's tiny band replays the same draws separately, because its shared database gate merges with the polynomial fallbacks in aggregate coverage. Denominator: finite inputs, including those outside the function's domain — f32 univariate exhaustive (every finite bit pattern); f32 bivariate and trivariate 1,073,741,824, f64 1,073,741,824 and f128 268,435,456 representation-uniform finite samples. `ldexp*` draws its integer exponent in `-2200..=2200`, matching its benchmark band. `0` means no accurate-leg entries observed (exact only for an exhaustive sweep or an implementation with no accurate leg); other values are three significant digits of the percentage.

**Accurate cycles (v3).** llvm-mca cycles of one static path from the accurate leg's entry. Conditional branches fall through except explicit `acc_take` entries in `FN`; loops count once and local calls expand. This omits repeated iterations and other branches; jump tables choose their first arm. `incomplete` marks a path the walker could not finish. Opaque calls use llvm-mca's flat cost, so the estimate is neither a measured cost nor a guaranteed lower bound.

**Precision.** The working precision of the fast leg and of the accurate leg, hand-maintained in `tools/analysis.py` (`FN`). Arrows separate successive tiers; `or` denotes alternative bands. `?` means unrecorded.

**Table bytes.** Every read-only symbol a memory operand on either path references (`sym(%rip)`, `sym(,%reg,8)`, `.LCPI*`, `.Lanon*`), sized from the data directives after its label in the full assembly and summed once per matching set of emitted data directives: rustc emits a `const` table once per codegen unit, so the copy the inlined fast path reads and the copy the library's accurate leg reads are byte-identical and count once, as do equal constants behind different local labels. The result is the constant footprint the two paths can touch, whether or not a given input reads all of it.

## Calibration

Criterion medians of the corresponding functions, beside the representative native path's llvm-mca estimate. Criterion draws from each benchmark's band, which can execute other paths, and includes the random draw; the estimate does not.

| Host | Function | metallic ns | CORE-MATH ns | metallic cycles (native) | CORE-MATH cycles (native) |
| --- | --- | ---: | ---: | ---: | ---: |
| AMD Ryzen 7 8700F 8-Core Processor (2026-09-10, znver4) | `expf` | 13.05 | 13.63 | 57 | 55 |
| AMD Ryzen 7 8700F 8-Core Processor (2026-09-10, znver4) | `exp` | 17.39 | 16.42 | 70 | 59 |
| AMD Ryzen 7 8700F 8-Core Processor (2026-09-10, znver4) | `expq` | 55.45 | 55.86 | 145 | 134 |

AMD Ryzen 7 8700F 8-Core Processor: 16 logical CPUs; 1-minute load 18.79 before / 19.36 after; 26.5 GiB available memory before. Timings under load are context, not an idle-host calibration.

## Binary32

| Function | v3 | v4 | native (znver4) | FMA calls @v2 | Accurate leg % | Accurate cycles (v3) | Precision | Table bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: |
| `acosf` | 63 / 58 | 59 / 54 | 62 / 55 | 16 / 0 | 0.00434 / 0.145 | 61 / 45 | f64 → f64 + exceptions / f64 → f64 + exceptions | 292 / 248 |
| `acoshf` | 77 / 109† | 81 / 111† | 81 / 110† | 11 / 0 | 0 / 0.0243 | — / 50 | f64 / f64 → f64 | 1712 / 2176 |
| `acospif` | 47 / 116† | 48 / 117† | 47 / 111† | 8 / 0 | 0 / 0 | — / — | f64 / f64 | 1040 / 1076 |
| `asinf` | 58 / 59 | 52 / 53 | 58 / 60 | 17 / 0 | 0.0579 / 0.15 | 72 / 43 | f64 → f64 + exceptions / f64 → f64 + exceptions | 288 / 232 |
| `asinhf` | 89 / 109† | 90 / 112† | 93 / 108† | 17 / 0 | 0 / 0.0494 | — / 51 | f64 / f64 → f64 | 1768 / 2200 |
| `asinpif` | 47 / 115† | 47 / 115† | 48 / 110† | 8 / 1 | 0 / 0 | — / — | f64 / f64 | 1064 / 1068 |
| `atan2f` | 109 / 102 | 100 / 83 | 81 / 87 | 17 / 0 | 0.000406 / 2.51e-05 | 281 / 59 | f64 → double-double / f64 → f64 or double-double | 488 / 208 |
| `atan2pif` | 114 / 101 | 105 / 85 | 88 / 87 | 17 / 0 | 0.000375 / 49.1 | 304 / 178 | f64 → double-double / f64 → double-double | 504 / 760 |
| `atanf` | 86 / 88 | 70 / 74 | 68 / 73 | 13 / 0 | 0 / 0 | — / — | f64 / f64 | 152 / 144 |
| `atanhf` | 70 / 64 | 70 / 61 | 70 / 58 | 11 / 0 | 0.00583 / 0.0408 | 104 / 54 | f64 → f64 / f64 → f64 | 1768 / 1336 |
| `atanpif` | 88 / 86 | 72 / 73 | 70 / 69 | 13 / 0 | 0 / 0 | — / — | f64 / f64 | 160 / 128 |
| `cbrtf` | 98 / 58 | 87 / 53 | 75 / 55 | 2 / 0 | 0 / 1.52 | — / 32 | f64 / f64 → f64 | 16 / 112 |
| `compoundf` | 133 / 148 | 120 / 158 | 124 / 156 | 21 / 11 | 0.0544 / 0.036 | 53 / 186 | f64 → exact integer or double-double / f64 → double-double | 300 / 3456 |
| `cosf` | 77 / 55 | 73 / 53 | 71 / 48 | 14 / 0 | 0 / 3.18e-06 | — / 15‡ | f64 / f64 → f32 exception table | 152 / 396 |
| `coshf` | 81 / 66 | 73 / 63 | 72 / 60 | 13 / 0 | 0 / 0.00605 | — / 53‡ | f64 / f64 → f64 | 128 / 376 |
| `cospif` | 47 / 45 | 47 / 44 | 41 / 40 | 5 / 0 | 0 / 0 | — / — | f64 / f64 | 1080 / 1072 |
| `erfcf` | 93 / 87 | 84 / 77 | 82 / 90 | 12 / 0 | 8.27e-05 / 0 | 158 / — | f64 → double-double / f64 | 14028 / 1364 |
| `erff` | 94 / 55 | 87 / 57 | 80 / 57 | 11 / 0 | 2.73e-05 / 0 | 158 / — | f64 → double-double / f64 | 14016 / 3648 |
| `exp2f` | 51 / 53 | 49 / 51 | 44 / 54 | 10 / 0 | 0 / 0.0231 | — / 35‡ | f64 / f64 → f64 + exceptions | 104 / 612 |
| `exp2m1f` | 61 / 53 | 58 / 53 | 59 / 46 | 5 / 0 | 0.000284 / 0 | 77 / — | f64 → double-double + exceptions → 128-bit / f64 | 2488 / 192 |
| `exp10f` | 60 / 58 | 57 / 55 | 53 / 53 | 12 / 0 | 0 / 0.0122 | — / 48‡ | f64 / f64 → f64 | 128 / 376 |
| `exp10m1f` | 61 / 59 | 58 / 55 | 59 / 49 | 5 / 0 | 0.000305 / 0 | 77 / — | f64 → double-double + exceptions → 128-bit / f64 + exact-case table | 2496 / 216 |
| `expf` | 64 / 55 | 60 / 54 | 57 / 55 | 13 / 0 | 0 / 0.0153 | — / 43‡ | f64 / f64 → f64 | 128 / 632 |
| `expm1f` | 60 / 59 | 57 / 58 | 59 / 57 | 5 / 0 | 0.00031 / 0.00793 | 66 / 43 | f64 → f64 / f64 → f64 | 464 / 384 |
| `fmaf` | 10‡ / — | 10‡ / — | 8‡ / — | 1 / — | 0 / — | — / — | f32 FMA / — | 0 / — |
| `frexpf` | 16 / — | 17 / — | 12 / — | 0 / — | 0 / — | — / — | 32-bit integer / — | 0 / — |
| `hypotf` | 54 / 117† | 59 / 116† | 54 / 109† | 0 / 0 | 5.59e-07 / 5.59e-07 | 63 / 56 | f64 → double-double / f64 → double-double | 20 / 20 |
| `ldexpf` | 18‡ / — | 18‡ / — | 13‡ / — | 0 / — | 0 / — | — / — | f64 / — | 0 / — |
| `lgammaf` | 83 / 83 | 76 / 68 | 70 / 70 | 13 / 0 | 0.0112 / 9.19e-06 | 1703† / 18 | f64 → double-double / f64 → f32 exception table | 3800 / 788 |
| `log1pf` | 67 / 58 | 66 / 54 | 65 / 57 | 17 / 0 | 0 / 0.00893 | — / 74 | f64 / f64 → f64 pairs | 1812 / 664 |
| `log2f` | 50 / 44 | 50 / 43 | 49 / 44 | 10 / 0 | 0 / 0.0179 | — / 34‡ | f64 / f64 → f64 | 1696 / 1128 |
| `log2p1f` | 71 / 54 | 65 / 54 | 65 / 58 | 18 / 0 | 0 / 0.0674 | — / 35‡ | f64 / f64 → f64 + exceptions | 1776 / 1144 |
| `log10f` | 51 / 48 | 50 / 45 | 52 / 49 | 10 / 0 | 0 / 0.0178 | — / 62 | f64 / f64 → f64 | 1712 / 1224 |
| `log10p1f` | 64 / 59 | 64 / 56 | 63 / 57 | 11 / 0 | 0 / 43.7 | — / 60 | f64 / f64 → f64 + exceptions | 1728 / 1240 |
| `logf` | 59 / 46 | 53 / 43 | 52 / 44 | 10 / 0 | 0 / 0.0178 | — / 67 | f64 / f64 → f64 | 1660 / 1180 |
| `powf` | 125 / 100 | 117 / 100 | 121 / 99 | 21 / 2 | 0.0367 / 2.21e-05 | 590 / 104 | f64 → double-double / f64 → double-double | 864 / 1624 |
| `roundf` | 28‡ / — | 44‡ / — | 36‡ / — | 0 / — | 0 / — | — / — | f32 / — | 12 / — |
| `rsqrtf` | 52 / 110† | 44 / 109† | 36 / 107† | 0 / 0 | 0 / 0 | — / — | f64 / f64 | 8 / 8 |
| `sincosf` | 81 / 65 | 75 / 63 | 73 / 57 | 14 / 0 | 0 / 3.23e-06 | — / 15‡ | f64 / f64 → f32 exception table | 156 / 344 |
| `sinf` | 78 / 56 | 74 / 53 | 72 / 49 | 14 / 0 | 0 / 0 | — / — | f64 / f64 | 144 / 336 |
| `sinhf` | 75 / 66 | 75 / 63 | 75 / 60 | 13 / 0 | 0 / 0.00681 | — / 53‡ | f64 / f64 → f64 | 132 / 376 |
| `sinpif` | 48 / 46 | 49 / 45 | 43 / 43 | 5 / 0 | 0 / 0 | — / — | f64 / f64 | 1080 / 1072 |
| `tanf` | 92 / 82 | 82 / 74 | 77 / 68 | 8 / 0 | 0 / 1.73e-06 | — / 16‡ | f64 / f64 → f32 exception table | 96 / 192 |
| `tanhf` | 102 / 59 | 98 / 50 | 97 / 51 | 13 / 0 | 0 / 0 | — / — | f64 / f64 | 144 / 120 |
| `tanpif` | 75 / 65 | 73 / 60 | 56 / 51 | 0 / 0 | 0 / 0 | — / — | f64 / f64 | 76 / 76 |
| `tgammaf` | 137 / 61 | 124 / 58 | 104 / 58 | 14 / 0 | 0.0915 / 5.59e-06 | 316 / 14‡ | f64 → double-double / f64 → f32 exception table | 688 / 324 |

## Binary64

| Function | v3 | v4 | native (znver4) | FMA calls @v2 | Accurate leg % | Accurate cycles (v3) | Precision | Table bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: |
| `acos` | 95 / 113† | 87 / 117† | 85 / 114† | 12 / 2 | 0.000609 / 0.000465 | 60 / 227† | double-double → double-double → triple-double / double-double → double-double + exceptions | 2272 / 2912 |
| `acosh` | 103 / 112† | 101 / 119† | 108 / 116† | 13 / 3 | 0.000355 / 0.000726 | 338 / 265† | double-double → 128-bit / f64 or double-double → double-double or triple-double + exceptions | 12512 / 4040 |
| `acospi` | 100 / 92 | 91 / 89 | 89 / 86 | 16 / 8 | 0.000607 / 0.0185 | 70 / 44 | double-double → double-double → triple-double / double-double → double-double + exceptions | 2304 / 20442 |
| `asin` | 95 / 113† | 87 / 117† | 87 / 114† | 12 / 2 | 0.00311 / 0.00177 | 29 / 212† | double-double → double-double → triple-double / double-double → double-double + exceptions | 2240 / 2912 |
| `asinh` | 109 / 118† | 112 / 120† | 111 / 117† | 13 / 3 | 0.00591 / 0.00323 | 338 / 233 | f64 or double-double → double-double or 128-bit / f64 or double-double → double-double or triple-double + exceptions | 12520 / 4064 |
| `asinpi` | 100 / 135† | 91 / 132† | 91 / 139† | 16 / 1 | 6.01 / 0.00908 | 42 / 252† | double-double → double-double → triple-double or 128-bit / 64/128-bit fixed → 128-bit fixed | 2264 / 2128 |
| `atan` | 99 / 89 | 88 / 79 | 88 / 82 | 5 / 1 | 0.18 / 0.18 | 432 / 195 | f64 → double-double → 128-bit / f64 → double-double + exceptions | 3424 / 2458 |
| `atan2` | 147 / 134† | 137 / 122† | 140 / 118† | 6 / 2 | 2.77 / 95.1 | 40 / 139† | f64 → double-double → 192-bit / f64 pair → double-double → 192-bit | 2992 / 1848 |
| `atan2pi` | 152 / 169 | 141 / 167 | 130 / 160 | 10 / 16 | 2.78 / 70.3 | 99 / 870 | f64 or double-double → double-double → 192-bit / double-double → 192-bit | 3024 / 8896 |
| `atanh` | 111 / 117 | 105 / 108 | 103 / 98 | 18 / 4 | 0.000659 / 0.000228 | 272 / 212 | double-double → double-double or 128-bit / double-double → double-double or triple-double + exceptions | 20208 / 4064 |
| `atanpi` | 123 / 119 | 110 / 114 | 110 / 109 | 9 / 2 | 6.21 / 0.181 | 443 / 83 | f64 or double-double → double-double → 128-bit / f64 or double-double → double-double + exceptions | 3456 / 2410 |
| `cbrt` | 135 / 119 | 128 / 106 | 112 / 100 | 6 / 2 | 0.000134 / 2.35e-05 | 125 / 58 | f64 or double-double → double-double / f64 or double-double → double-double + exceptions | 168 / 360 |
| `compound` | 188 / — | 185 / — | 177 / — | 28 / — | 23.2 / — | 243 / — | double-double → 128-bit → exact integer or 256-bit / — | 13784 / — |
| `cos` | 101 / 122 | 94 / 124 | 83 / 117 | 21 / 20 | 1.3 / 0.0117 | 163 / 374† | f64 or double-double → 128-bit / double-double → 128-bit + exceptions | 856 / 22864 |
| `cosh` | 163 / 81 | 169 / 79 | 149 / 77 | 15 / 3 | 0.00219 / 0.00527 | 59 / 98 | double-double → double-double + exceptions / f64 or double-double → double-double + exceptions | 2328 / 2272 |
| `cospi` | 78 / 90 | 80 / 85 | 75 / 84 | 5 / 0 | 0.0563 / 0.0563 | 260 / 178 | f64 or double-double → double-double → 128-bit / f64 or double-double → double-double + exceptions | 2400 / 3304 |
| `erf` | 88 / 110 | 87 / 114 | 75 / 104 | 11 / 12 | 0.0112 / 9.78e-05 | 284† / 18 | double-double → double-double / double-double → double-double + exceptions | 12680 / 9968 |
| `erfc` | 216 / 101 | 215 / 106 | 196 / 94 | 45 / 10 | 0.00253 / 0.00331 | 52 / 105 | double-double → double-double + exceptions / double-double → double-double + exceptions | 2632 / 10128 |
| `exp` | 71 / 64 | 72 / 64 | 70 / 59 | 9 / 1 | 0.01 / 0.00605 | 165 / 154 | f64 pair → double-double + exceptions / f64 pair → double-double + exceptions | 3072 / 2208 |
| `exp2` | 71 / 67 | 72 / 66 | 70 / 59 | 9 / 1 | 0.00959 / 0.00611 | 163 / 90 | f64 pair → double-double + exceptions / f64 pair → double-double + exceptions | 2448 / 2208 |
| `exp2m1` | 77 / 122 | 76 / 121 | 76 / 109 | 9 / 15 | 6.01 / 0.00053 | 159 / 135 | f64 pair or double-double → double-double + exceptions or 128-bit / double-double → double-double + exceptions | 2296 / 3720 |
| `exp10` | 71 / 65 | 72 / 65 | 70 / 60 | 9 / 1 | 0.00979 / 0.00606 | 165 / 137 | f64 pair → double-double + exceptions / f64 pair → double-double + exceptions | 2472 / 2216 |
| `exp10m1` | 77 / 121 | 76 / 119 | 75 / 108 | 9 / 15 | 6.01 / 0.000115 | 163 / 151 | f64 pair or double-double → double-double + exceptions or 128-bit / double-double → double-double + exceptions | 2432 / 5800 |
| `expm1` | 77 / 68 | 75 / 70 | 75 / 63 | 9 / 1 | 0.000899 / 0.2 | 155 / 210 | f64 pair or double-double → double-double + exceptions / f64 pair → double-double + exceptions | 2344 / 2232 |
| `fma` | 10‡ / — | 10‡ / — | 8‡ / — | 1 / — | 0 / — | — / — | f64 FMA / — | 0 / — |
| `frexp` | 17 / — | 16 / — | 14 / — | 0 / — | 0 / — | — / — | 64-bit integer / — | 0 / — |
| `hypot` | 106 / 125† | 98 / 121† | 87 / 116† | 3 / 3 | 0 / 0 | 58 / 111† | f64 or double-double → exact 128-bit integer / f64 or double-double → exact 128-bit integer | 48 / 32 |
| `ldexp` | 12‡ / — | 12‡ / — | 10‡ / — | 0 / — | 0 / — | — / — | f64 / — | 0 / — |
| `lgamma` | 130 / 135 | 136 / 138 | 137 / 134 | 16 / 4 | 31.6 / 0.0154 | 4340† / 636 | double-double → double-double → triple-double / double-double → triple-double + exceptions | 7096 / 8752 |
| `log` | 68 / 78 | 65 / 74 | 64 / 80 | 10 / 7 | 0.00144 / 4.68e-05 | 240 / 542 | f64 or double-double → 128-bit / double-double → 128-bit | 20096 / 24664 |
| `log1p` | 91 / 88 | 83 / 85 | 87 / 83 | 11 / 1 | 0.0108 / 0.00351 | 82 / 198 | f64 or double-double → double-double or 128-bit / double-double → double-double or triple-double | 4344 / 3776 |
| `log2` | 65 / 79 | 66 / 75 | 64 / 72 | 11 / 1 | 0.00141 / 2.79e-06 | 265 / 201 | f64 or double-double → 128-bit / double-double → double-double or triple-double | 20160 / 4000 |
| `log2p1` | 90 / 124 | 86 / 122 | 88 / 124 | 14 / 11 | 6.02 / 5.31 | 263 / 19 | f64 or double-double → 128-bit / double-double → double-double + exceptions or 128-bit | 12488 / 9344 |
| `log10` | 75 / 79 | 71 / 74 | 71 / 82 | 13 / 10 | 0.00307 / 6.38e-05 | 265 / 562 | f64 or double-double → 128-bit / double-double → 128-bit | 20176 / 24896 |
| `log10p1` | 90 / 118 | 86 / 118 | 88 / 118 | 14 / 11 | 6.02 / 5.41 | 263 / 51 | f64 or double-double → 128-bit / double-double → double-double + exceptions or 128-bit | 12488 / 8912 |
| `pow` | 183 / 149 | 176 / 131 | 166 / 132 | 27 / 26 | 0.00186 / 1.81 | 18 / 347 | double-double → 128-bit → exact integer or 256-bit / double-double → 128-bit → exact integer or 256-bit | 6400 / 21096 |
| `round` | 28‡ / — | 44‡ / — | 36‡ / — | 0 / — | 0 / — | — / — | f64 / — | 24 / — |
| `rsqrt` | 85 / 108† | 68 / 112† | 73 / 109† | 2 / 2 | 0 / 0 | 51 / 51 | f64 or double-double → exact 128-bit integer / f64 or double-double → exact 128-bit integer | 24 / 32 |
| `sin` | 94 / 120 | 98 / 127 | 81 / 116 | 17 / 20 | 1.28 / 0.593 | 179 / 387† | f64 or double-double → 128-bit / double-double → 128-bit + exceptions | 848 / 22736 |
| `sincos` | 117 / 129 | 111 / 132 | 91 / 123 | 31 / 24 | 2.55 / 0.665 | 250 / 527 | f64 or double-double → 128-bit / double-double → 128-bit + exceptions | 1440 / 23112 |
| `sinh` | 168 / 86 | 175 / 83 | 153 / 94 | 15 / 3 | 0.0528 / 0.00808 | 23 / 114 | f64 or double-double → double-double + exceptions / f64 or double-double → double-double + exceptions | 2224 / 2240 |
| `sinpi` | 79 / 89 | 83 / 85 | 81 / 82 | 4 / 0 | 0.0694 / 0.0688 | 219 / 178 | f64 or double-double → double-double → 128-bit / f64 or double-double → double-double + exceptions | 1904 / 3304 |
| `tan` | 149 / 148 | 138 / 147 | 124 / 132 | 41 / 26 | 1.13 / 0.00995 | 449 / 553† | double-double → 128-bit / double-double → 128-bit + exceptions | 1504 / 24816 |
| `tanh` | 132 / 139 | 126 / 130 | 118 / 126 | 11 / 4 | 0.0171 / 0.0129 | 23 / 165 | f64 or double-double → double-double + exceptions / f64 or double-double → double-double + exceptions | 2224 / 2256 |
| `tanpi` | 119 / 122 | 115 / 115 | 98 / 107 | 6 / 4 | 0.000648 / 0.000524 | 352 / 272 | double-double → 128-bit / double-double → double-double + exceptions | 1592 / 728 |
| `tgamma` | 122 / 198 | 129 / 181 | 126 / 166 | 13 / 11 | 0.157 / 0.0237 | 2521† / 116 | double-double → triple-double / double-double → double-double + exceptions | 36048 / 3600 |

## Binary128

| Function | v3 | v4 | native (znver4) | FMA calls @v2 | Accurate leg % | Accurate cycles (v3) | Precision | Table bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: |
| `acosq` | 407 / 292 | 372 / 268 | 342 / 284 | 0 / 0 | 0.0014 / 0 | 1040† / 1092† | 128-bit fixed (256-bit frame) → 384-bit fixed / 128-bit fixed (192-bit frame) → 320-bit fixed | 11688 / 8110 |
| `asinq` | 407 / 285 | 371 / 262 | 344 / 264 | 0 / 0 | 0.00379 / 0 | 1040† / 1087† | 128-bit fixed (256-bit frame) → 384-bit fixed / 128-bit fixed (192/256-bit frame) → 320-bit fixed | 11688 / 8062 |
| `atan2q` | 313† / 251† | 273† / 235† | 233† / 269† | 0 / 0 | 6.32 / 0.0137 | 756† / 855† | 128-bit fixed (256-bit frame) → 384-bit fixed / 128-bit fixed (192-bit frame) → 384-bit fixed | 4272 / 4403 |
| `atanq` | 353† / 199† | 303† / 183† | 219† / 200† | 0 / 0 | 0.00514 / 7.79e-05 | 756† / 809† | 128-bit fixed (256-bit frame) → 384-bit fixed / 128-bit fixed (192-bit frame) → 384-bit fixed | 4248 / 4130 |
| `cbrtq` | 130 / 127 | 129 / 124 | 127 / 160 | 0 / 0 | 3.17 / 1.56 | 120 / 55 | 128-bit fixed → exact 384-bit integer / 128-bit fixed → 256-bit or modular 128-bit integer | 1584 / 168 |
| `cosq` | 279 / — | 240 / — | 227 / — | 0 / — | 0.0504 / — | 1100 / — | 128-bit fixed (256-bit frame) → 384-bit fixed / — | 16152 / — |
| `exp2q` | 158 / 97 | 143 / 84 | 145 / 95 | 0 / 0 | 0.000777 / 0.00133 | 356 / 584 | 128-bit fixed → 256-bit fixed / 128-bit fixed → 192-bit fixed → 384-bit fixed | 6632 / 5720 |
| `exp10q` | 158 / 105 | 143 / 91 | 145 / 102 | 0 / 0 | 0.000771 / 0.00132 | 356 / 633 | 128-bit fixed → 256-bit fixed / 128-bit fixed → 192-bit fixed → 384-bit fixed | 6632 / 5720 |
| `expm1q` | 167 / 129 | 155 / 107 | 146 / 163 | 0 / 0 | 0.00132 / 0.000178 | 368 / 140 | 128-bit fixed → 256-bit fixed (384-bit product) / 128-bit fixed → 192-bit fixed → 384-bit fixed | 6632 / 3176 |
| `expq` | 158 / 102 | 143 / 90 | 145 / 134 | 0 / 0 | 0.000801 / 0.00134 | 356 / 133 | 128-bit fixed → 256-bit fixed / 128-bit fixed → 192-bit fixed → 384-bit fixed | 6632 / 3176 |
| `hypotq` | 177 / 149 | 161 / 138 | 144 / 98 | 0 / 0 | 0.00282 / 0.000288 | 328† / 104 | 128-bit fixed → exact 384-bit integer / 128-bit fixed → 256-bit or modular 128-bit integer | 1624 / 1040 |
| `log1pq` | 191 / — | 171 / — | 172 / — | 0 / — | 0.000608 / — | 1165† / — | 128/256-bit fixed → 256/384-bit fixed / — | 12564 / — |
| `log2q` | 144 / — | 126 / — | 130 / — | 0 / — | 0 / — | 885† / — | 256-bit fixed → 384-bit fixed / — | 12556 / — |
| `log10q` | 165 / — | 148 / — | 146 / — | 0 / — | 0 / — | 979† / — | 256-bit fixed → 384-bit fixed / — | 12564 / — |
| `logq` | 162 / 151 | 134 / 145 | 140 / 142 | 0 / 0 | 0 / 0 | 1165† / 517 | 256-bit fixed → 384-bit fixed / 128-bit fixed (192-bit frame) → 384-bit fixed (448-bit product) | 12564 / 6896 |
| `powq` | 283 / — | 256 / — | 254 / — | 0 / — | 0.000431 / — | 562 / — | 128/256-bit fixed → 256/384-bit fixed → exact integer or 640-bit fixed / — | 19116 / — |
| `rsqrtq` | 117 / 97 | 113 / 93 | 102 / 133 | 0 / 0 | 1.59 / 0.0973 | 117 / 53 | 128-bit fixed → exact 384-bit integer / 128-bit fixed → exact 256-bit integer | 1576 / 1040 |
| `sinq` | 279 / — | 240 / — | 227 / — | 0 / — | 0.0506 / — | 1100 / — | 128-bit fixed (256-bit frame) → 384-bit fixed / — | 16152 / — |
| `sqrtq` | 143 / 97 | 135 / 93 | 116 / 90 | 0 / 0 | 0.397 / 0.0396 | 69 / 27 | 128-bit fixed → exact 256/384-bit integer / 128-bit fixed → modular 128-bit integer | 1576 / 1040 |
| `tanq` | 360† / — | 313† / — | 292† / — | 0 / — | 0.198 / — | 1238† / — | 128-bit fixed (256-bit frame) → 384-bit fixed / — | 9480 / — |

## Caveats

- llvm-mca's AMD models have historically been the weaker ones; the Zen model here is the closest to the host, not a measurement of it, and the calibration rows above are the only check on its scale. Its znver3/znver4 register-file model also loses a loop-carried dependency when a zero idiom hits the source of an eliminated move, which is why the columns are single-pass latencies rather than the steady state of a dependent loop: the single pass includes the front-end fill of the block, a few cycles for a short path and more for a long one.
- A `call` on a path is priced at 100 cycles by llvm-mca whatever the callee costs; rows marked `†` are estimates of the wrapper, not of the callee.
- The accurate-leg fraction is a property of the input distribution: representation-uniform finite inputs, exhaustive for f32 univariate functions when `--f32-samples=0`. A workload concentrated near a function's hard cases pays the accurate leg far more often.
- Table loads are assumed to hit L1; the table-bytes column says how much must be resident for that to hold. It counts byte-identical data once, while the linked binary holds rustc's per-codegen-unit copies of a `const` table separately (the example's inlined fast path and the library's accurate leg each read their own), so a crate that inlines the fast path carries more than the column says.
- Criterion rows include the RNG draw of each input and the loop around the call.
- The representative input selects one path. Magnitude, sign, argument reduction and exact cases can select very different paths; use the existing Criterion benches for a workload comparison. Internal call/return overhead is omitted when the callee is expanded into the path.
