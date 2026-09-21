# Binary128 exp family on Intel P- and E-cores

This campaign re-measures the gap reported in
[issue #12](https://github.com/jdh8/metallic-rs/issues/12) (`expq`, `exp2q`,
`exp10q` at ~1.4× CORE-MATH). That sweep was pinned to CPU 30, which on this
i9-14900K host is an **E-core** (Gracemont). The gap is specific to the E-core.
On a P-core (Raptor Cove), metallic is about **15% faster** than CORE-MATH for
the same three functions.

No code changed. Both cores ran the same executables at `9f50225`, whose
`src/` matches the issue's `0f0e866`.

## Measurements

Median ns and same-run metallic / CORE-MATH ratios for round 1 and round 2.
Rounds were interleaved: each round measured the P-core, then the E-core.

| Function | P-core metallic | P-core CORE-MATH | P-core ratio | E-core metallic | E-core CORE-MATH | E-core ratio |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `expq` | 24.17 / 24.18 | 28.38 / 28.50 | **0.852 / 0.848** | 63.40 / 62.79 | 44.52 / 44.41 | 1.424 / 1.414 |
| `exp2q` | 23.98 / 23.98 | 28.19 / 28.32 | **0.851 / 0.847** | 61.58 / 61.68 | 42.97 / 42.97 | 1.433 / 1.436 |
| `exp10q` | 24.32 / 24.32 | 28.73 / 28.71 | **0.847 / 0.847** | 62.13 / 61.50 | 44.55 / 44.54 | 1.395 / 1.381 |
| `expm1q` | 29.76 / 29.75 | 32.43 / 32.45 | **0.918 / 0.917** | 58.12 / 59.28 | 53.64 / 53.82 | 1.084 / 1.102 |

A replication pass with identical settings ran immediately before these rounds.
Its ratios, stored as `replication_pass_ratios` in [results.json](results.json),
agree within 1% on the P-core and 3.5% on the E-core.

CORE-MATH's time roughly doubles when moving from the P-core to the E-core
(×1.52–1.66). Metallic's time more than doubles for all four functions
(×1.95–2.62), and most for the three functions that share `exp.rs::fast`.
That fast leg is one serial chain of 128-bit integer products: the Horner steps,
then three table products. A dependent-latency or multiplier-throughput
difference on Gracemont is the likely cause, but this campaign does not
confirm it. Checking would take disassembly and a per-core llvm-mca model.

`ANALYSIS.md` prices the fast leg at 158 cycles against CORE-MATH's 97–105,
using `llvm-mca -mcpu=x86-64-v3`. That static ratio (1.5–1.6×) is close to the
E-core measurement and nothing like the P-core one. It is therefore not portable
evidence of a gap on every host, as issue #12 argued. It reflects that
scheduling model, which is neither of these cores.

## Conditions

Intel Core i9-14900K. The container's cgroup allows only CPUs 4-5 and 30-31.
CPU 4 is a P-core (5.7 GHz max) whose SMT sibling, CPU 5, stayed online:
without root access in the container it could not be taken offline, and
nothing was pinned to it. CPU 30 is an E-core (4.4 GHz max, no SMT).

- Toolchain: rustc 1.100.0-nightly (`a36d05efa`, 2026-09-09) and Ubuntu clang
  21.1.8. Library versions: core-math and core-math-sys 1.3.0, Criterion 0.8.2.
- Compiler flags: `CC=clang` and `RUSTFLAGS=-Ctarget-cpu=x86-64-v3`. This
  overrides the local `~/.cargo/config.toml`, which sets `target-cpu=native`.
  The C oracle uses core-math-sys's default `-march=native`.
- Criterion: 1 s warm-up, 2 s measurement, 100 samples, 100,000 bootstrap
  resamples.
- Sampler: the existing `bench::Exponents` samplers. Inputs are signed, with
  the exponent uniform in `-120..=13` (`-114..=13` for `expm1q`). Random-input
  generation is timed.
- Runs were serial. No tests or other benchmarks ran alongside them.
- An editor, rust-analyzer and agent sessions stayed idle in the background.
  The load average was 1.0–1.3, CPU and memory PSI avg10 were 0.00, and about
  4.3 GB of 16 GB memory was available. [timing.log](timing.log) records these
  per run.

Source, executable and dependency-lock hashes are in [results.json](results.json);
[Cargo.lock.snapshot](Cargo.lock.snapshot) preserves the dependencies.

Reproduce one lane:

```sh
CC=clang RUSTFLAGS=-Ctarget-cpu=x86-64-v3 taskset -c 4 cargo +nightly bench \
  --locked --features f128 --bench expq -- '^(metallic|core_math)::' \
  --warm-up-time 1 --measurement-time 2 --sample-size 100 \
  --nresamples 100000 --noplot
```

Use `taskset -c 30` for the E-core. Check `lscpu -e` or
`/sys/devices/system/cpu/cpu*/cpufreq/cpuinfo_max_freq` first: CPU numbering of
P- and E-cores differs between hosts.

Only this host was measured. Zen 4 and Apple M4 were not checked in this
campaign.
