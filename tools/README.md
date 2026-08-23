# tools

Table/constant generators (Python 3 + mpmath unless noted) and maintenance
scripts.  Every generated table in `src/` carries a provenance header naming its
generator — regenerate with the script, never edit the table by hand.

| Script | What it emits | Consumed by |
| --- | --- | --- |
| `bench_ratio.py` | metallic-vs-CORE-MATH ratio table from `target/criterion/*/new/estimates.json` (arg: `mean`/`median`) | perf triage (issue #5) |
| `gen_atan_f64.py` | `ATAN_COEFFS`, `ATAN_TABLE` (atan(k/8) double-double) | `src/f64_/atan.rs` |
| `gen_atan_fast_f64.py` | `ATAN_FAST_*` fast-path tables (from CORE-MATH `atan.c` hex floats) | `src/f64_/atan.rs` |
| `gen_dint_atan.py` | 128-bit `Dint` accurate-path atan constants | `src/f64_/atan.rs` |
| `gen_dint_trig.py` | `Dint` accurate-path trig constants (minimax sin/cos) | `src/f64_/trig.rs` |
| `gen_erf_f64.py` | `erf`/`erfc` minimax + domain-split segments + `ERF_TABLE` cells | `src/f64_/erf.rs` |
| `gen_erf_hard.py` | `ERFC_HARD` hard-to-round exception database | `src/f64_/erf.rs` |
| `gen_exp_f128.py` | binary128 exp family: 2^(j/2^k) tables, `ln(2)^k/k!`, log2(e)/log2(10) limbs | `src/f128_/exp_tables.rs` |
| `gen_exp_f64.py` | f64 `exp` table + reduction + double-double poly | `src/f64_/exp.rs` |
| `gen_gamma_f64.py` | `tgamma` central minimax + recurrence + triple-double tail | `src/f64_/gamma.rs` |
| `gen_inv_f64.py` | `ASIN_CELLS` (64-byte cells, per-cell Chebyshev, Ziv gate constants) | `src/f64_/atan.rs` |
| `gen_lgamma_td.py` | triple-double `lgamma` accurate-path tables (`--inline` for consts) | `src/f64_/gamma_td_tables.rs` |
| `gen_ln_exact_f64.py` | exact-`z` reduction cells for the `ln` fast leg | `src/f64_/log.rs` |
| `gen_log2_exact_f64.py` | exact-`z` cells for the native base-2 `log2` fast leg | `src/f64_/log.rs` |
| `gen_log_f128.py` | binary128 `log`: reciprocal + logarithm tables, `ln(2)` limbs, `1/(k+1)`, crude-log2 fit | `src/f128_/log_tables.rs` |
| `gen_log_f64.py` | accurate-tier constants for the log family | `src/f64_/log.rs` |
| `gen_pow_tables.py` | verbatim `Dint`/`Qint` literals from CORE-MATH `dint.h`/`qint.h` | `src/f64_/pow_consts.rs` |
| `gen_trig_f64.py` | sin/cos/tan kernels, Cody–Waite π/2 words, Payne–Hanek 2/π | `src/f64_/trig.rs` |
| `sync-worst-cases.sh` | refreshes `tests/cases/*.wc` from a `core-math-sys` checkout (`CORE_MATH_SYS` env) | `tests/*.rs` worst-case gates |
