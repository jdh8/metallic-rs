#!/usr/bin/env python3
"""Generate BENCHMARKS.md from complete, independently archived host snapshots.

    python3 tools/benchmark_report.py --snapshots benchmarks/2026-09-05

The three required directories are local, dl02, and dl02-f128. Every measured
lane must be represented in the report, and the public API and source hashes
must agree with the snapshots. Requires Python 3.9+; no external packages.
"""

import argparse
import hashlib
import json
import math
from pathlib import Path
import re
import shlex


ROOT = Path(__file__).resolve().parent.parent
EXTRA_BANDS = {
    "tgamma": ("reflect", "recur", "stirling"),
    "lgamma": ("reflect", "recur", "stirling"),
    "lgammaf": ("reflect", "rational", "stirling"),
}
STD_ALIASES = {"log": "ln", "log1p": "ln_1p", "expm1": "exp_m1",
               "sincos": "sin_cos", "pow": "powf"}
NO_CORE_DEFAULT = {"compound", "fma", "fmaf", "frexp", "frexpf", "ldexp",
                   "ldexpf", "round", "roundf"}
NO_CORE_F128 = {"cosq", "sinq", "tanq", "log2q", "log10q", "log1pq", "powq"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def link(path):
    return Path(path).resolve().relative_to(ROOT).as_posix()


def cell(value):
    return str(value).replace("|", "\\|").replace("\n", "<br>")


def output_of(probe):
    return probe.get("output", "unavailable")


def first_line(probe):
    return output_of(probe).splitlines()[0]


def public_api():
    groups = {"f32_": set(), "f64_": set(), "f128_": set()}
    pattern = r"pub use (f32_|f64_|f128_)(?:::\w+)*::\{([^}]+)\};"
    for group, body in re.findall(pattern, (ROOT / "src/lib.rs").read_text()):
        groups[group].update(name.strip() for name in body.split(",") if name.strip())
    for group, count in (("f32_", 46), ("f64_", 46), ("f128_", 20)):
        require(len(groups[group]) == count,
                "Public API changed: update the report's audited coverage for " + group)
    return groups


def load_snapshot(directory, precision, expected):
    path = directory / "results.json"
    data = json.loads(path.read_text())
    require(data.get("schema_version") == 1, "Unsupported snapshot schema: " + str(path))
    require(data.get("status") == "complete", "Incomplete snapshot: " + str(path))
    require(data.get("precision") == precision, "Wrong precision: " + str(path))
    require(set(data["expected_public_functions"]) == expected,
            "Public API mismatch: " + str(path))
    require(len(data["expected_public_functions"]) == len(expected),
            "Duplicated public functions: " + str(path))
    require(set(data["targets"]) == expected and len(data["targets"]) == len(expected),
            "Benchmark target coverage mismatch: " + str(path))
    require(not data["coverage"]["missing"], "Missing benchmark results: " + str(path))
    require(data["coverage"]["public_functions"] == len(expected),
            "Coverage count mismatch: " + str(path))
    lock = directory / "Cargo.lock.snapshot"
    require(hashlib.sha256(lock.read_bytes()).hexdigest() == data["source"]["cargo_lock_sha256"],
            "Archived Cargo.lock hash mismatch: " + str(lock))
    measures = data["measurements"]
    extras = {target + "_" + band for target, bands in EXTRA_BANDS.items() for band in bands}
    wanted = expected | (extras if precision == "default" else set())
    found = {ident.split("::", 1)[1] for ident in measures if ident.startswith("metallic::")}
    require(found == wanted,
            "Metallic measurement coverage mismatch in {}: missing {}, unexpected {}".format(
                path, sorted(wanted - found), sorted(found - wanted)))
    for ident, record in measures.items():
        require(record["target"] in expected, "Unexpected target for " + ident)
        require(record["benchmark"]["full_id"] == ident, "Mislabeled measurement: " + ident)
        require(record["sample_count"] == data["settings"]["sample_size"],
                "Unexpected Criterion sample count: " + ident)
        for statistic in ("mean", "median"):
            value = record["estimates"][statistic]["point_estimate"]
            require(isinstance(value, (int, float)) and math.isfinite(value) and value > 0,
                    "Invalid {} estimate: {}".format(statistic, ident))
    require(data["coverage"]["measurement_lanes"] == len(measures),
            "Measurement lane count mismatch: " + str(path))
    for command in data["commands"]:
        require(command["exit_code"] == 0, "Failed command in complete snapshot: " + str(path))
    data["_directory"] = directory
    data["_consumed"] = set()
    return data


def measured(data, target, ident):
    record = data["measurements"].get(ident)
    if record is None:
        return None
    require(record["target"] == target,
            "Cross-target measurement pairing: {} belongs to {}".format(ident, record["target"]))
    require(ident not in data["_consumed"], "Measurement used twice: " + ident)
    data["_consumed"].add(ident)
    return record["estimates"]["median"]["point_estimate"]


def number(value):
    return "—" if value is None else "{:.2f}".format(value)


def table(data, names, precision, extra=False):
    comparison = "libquadmath" if precision == "f128" else "libm"
    lines = ["| Function | metallic ns | CORE-MATH ns | M/CORE | std/system ns | " + comparison + " ns |",
             "| --- | ---: | ---: | ---: | ---: | ---: |"]
    absent_core = NO_CORE_F128 if precision == "f128" else NO_CORE_DEFAULT
    for name in sorted(names):
        target = name.split("_", 1)[0] if extra else name
        metallic = measured(data, target, "metallic::" + name)
        core = measured(data, target, "core_math::" + name)
        require(metallic is not None, "Missing metallic lane: " + name)
        require((core is None) == (target in absent_core), "CORE-MATH lane mismatch: " + name)
        standard = None
        if not extra:
            base = target[:-1] if precision in ("f32", "f128") else target
            method = STD_ALIASES.get(base, base)
            standard = measured(data, target, precision + "::" + method)
            if target == "exp10q":
                require(standard is None, "Two system lanes for exp10q")
                standard = measured(data, target, "glibc::exp10f128")
        other = measured(data, target, ("quadmath::" if precision == "f128" else "libm::") + name)
        ratio = "—" if core is None else "{:.2f}×".format(metallic / core)
        lines.append("| [{}](benches/{}.rs) | {} | {} | {} | {} | {} |".format(
            name, target, number(metallic), number(core), ratio, number(standard), number(other)))
    return "\n".join(lines)


def hardware(data):
    raw = output_of(data["machine"].get("hardware", {}))
    model = re.search(r"^(?:Model name|machdep\.cpu\.brand_string):\s*(.+)$", raw, re.M)
    return model.group(1).strip() if model else data["machine"]["machine"]


def rust_version(data):
    probe = data["machine"]["rustc"]
    llvm = re.search(r"^LLVM version:\s*(.+)$", output_of(probe), re.M)
    return first_line(probe) + ("; LLVM " + llvm.group(1) if llvm else "")


def memory_size(data):
    hardware_text = output_of(data["machine"].get("hardware", {}))
    mac = re.search(r"^hw\.memsize:\s*(\d+)$", hardware_text, re.M)
    if mac:
        return "{:.2f} GiB".format(int(mac.group(1)) / 2**30)
    memory = data["machine_start"].get("memory_kib", {})
    return "{:.2f} GiB".format(memory["MemTotal"] / 2**20) if "MemTotal" in memory else "unavailable"


def power_policy(data):
    machine = data["machine"]
    if "power" in machine:
        return first_line(machine["power"])
    policy = machine.get("frequency_policy", {})
    return "; ".join("{}={}".format(key, value) for key, value in sorted(policy.items())) or "unavailable"


def environment_value(data, name):
    value = data["environment"].get(name)
    return "unset" if value is None else (value or "empty")


def sampled_states(data):
    samples = [data.get("machine_before_measurement", data["machine_start"])]
    samples += [state for state in data["machine_samples"] if state.get("phase") != "build"]
    samples.append(data["machine_end"])
    return samples


def state_summary(data):
    samples = sampled_states(data)
    loads = [state["load_average"][0] for state in samples]
    parts = ["1-minute load {:.2f}–{:.2f}".format(min(loads), max(loads))]
    available = [state["memory_kib"]["MemAvailable"] / 2**20 for state in samples
                 if "memory_kib" in state and "MemAvailable" in state["memory_kib"]]
    if available:
        parts.append("available RAM {:.2f}–{:.2f} GiB".format(min(available), max(available)))
    swapped = [(state["memory_kib"]["SwapTotal"] - state["memory_kib"]["SwapFree"]) / 2**20
               for state in samples if "memory_kib" in state]
    if swapped:
        parts.append("maximum swap used {:.2f} GiB".format(max(swapped)))
    return "; ".join(parts)


def background_workloads(local, remote, quad):
    paragraphs = []
    ui = {}
    for state in sampled_states(local):
        for process in state.get("top_cpu", []):
            executable = process["executable"]
            if any(name in executable.lower() for name in ("windowserver", "codex", "chrome", "safari")):
                name = Path(executable).name
                ui[name] = max(ui.get(name, 0), process["cpu_percent"])
    if ui:
        examples = sorted(ui, key=lambda name: -ui[name])[:3]
        paragraphs.append("The local Mac had background UI activity in its process samples "
                          "({}).".format(", ".join("`{}`".format(name) for name in examples)))
    batch = []
    for data in (remote, quad):
        for state in sampled_states(data):
            batch.extend(process["cpu_percent"] for process in state.get("top_cpu", [])
                         if re.search(r"dump[-_]teacher", process["executable"], re.I))
    conditions = remote["_directory"] / "conditions.txt"
    evidence = conditions.read_text() if conditions.is_file() else ""
    if batch:
        scheduling = " in the `IDL` scheduling class" if "IDL" in evidence else ""
        citation = " ([recorded process conditions]({}))".format(link(conditions)) if evidence else ""
        paragraphs.append("On dl02, the concurrent `dump-teacher` batch{} persisted in the "
                          "process samples, reporting up to {:.1f}% CPU{}.".format(
                              scheduling, max(batch), citation))
    paragraphs.append("These background workloads can influence timing and within-run ratios; "
                      "interpret small differences cautiously. Full machine-state records "
                      "also retain memory/swap observations and the largest CPU and memory "
                      "consumers. The load ranges above cover observations during measurement.")
    return " ".join(paragraphs)


def environment_table(snapshots):
    lines = ["| Recorded setting | Local | dl02 f32/f64 | dl02 binary128 |",
             "| --- | --- | --- | --- |"]
    fields = [
        ("Host", lambda d: d["machine"]["hostname"]),
        ("CPU", hardware),
        ("Logical CPUs", lambda d: d["machine"]["logical_cpus"]),
        ("RAM", memory_size),
        ("OS / architecture", lambda d: d["machine"]["platform"]),
        ("Rust / LLVM", rust_version),
        ("Cargo", lambda d: first_line(d["machine"]["cargo"])),
        ("C compiler", lambda d: first_line(d["machine"]["c_compiler"])),
        ("RUSTFLAGS", lambda d: environment_value(d, "RUSTFLAGS")),
        ("CFLAGS", lambda d: environment_value(d, "CFLAGS")),
        ("TARGET_CPU", lambda d: environment_value(d, "TARGET_CPU")),
        ("Power / frequency policy", power_policy),
        ("CPU affinity", lambda d: d["settings"]["cpu_affinity"]
         if d["settings"]["cpu_affinity"] is not None else "OS scheduler"),
        ("Measurements started (UTC)", lambda d: d["measurement_start_utc"]),
        ("Run completed (UTC)", lambda d: d["end_utc"]),
        ("Sampled machine state", state_summary),
        ("Full metadata and estimates", lambda d: "[results.json]({})".format(
            link(d["_directory"] / "results.json"))),
    ]
    for label, getter in fields:
        lines.append("| {} | {} |".format(label, " | ".join(cell(getter(d)) for d in snapshots)))
    return "\n".join(lines)


def dependencies(lock_path):
    result = {}
    for section in lock_path.read_text().split("[[package]]")[1:]:
        name = re.search(r'^name = "([^"]+)"$', section, re.M)
        version = re.search(r'^version = "([^"]+)"$', section, re.M)
        if name and version and name.group(1) in ("core-math", "core-math-sys", "criterion", "libm", "rand"):
            result[name.group(1)] = version.group(1)
    require(len(result) == 5, "Missing expected dependencies in archived Cargo.lock")
    return ", ".join("`{} {}`".format(name, version) for name, version in sorted(result.items()))


def reproduce(data, label):
    env = {key: value for key, value in data["environment"].items()
           if key not in ("CARGO_TARGET_DIR", "CRITERION_HOME")}
    prefix = ["env"] + [key + "=" + value for key, value in sorted(env.items())]
    command = prefix + ["python3", "tools/benchmark_snapshot.py", "--precision", data["precision"],
                        "--target-dir", "target/benchmark-repeat-" + label,
                        "--output", "benchmarks/repeat/" + label]
    for key, option in (("warm_up_seconds", "--warm-up-time"),
                        ("measurement_seconds", "--measurement-time"),
                        ("sample_size", "--sample-size"), ("nresamples", "--nresamples")):
        command.extend((option, str(data["settings"][key])))
    if data["settings"]["cpu_affinity"] is not None:
        command.extend(("--cpu", str(data["settings"]["cpu_affinity"])))
    return "```sh\ncp {} Cargo.lock\n{}\n```".format(
        shlex.quote(link(data["_directory"] / "Cargo.lock.snapshot")), shlex.join(command))


STATIC_ESTIMATE = r"""## Static estimate

Cycle counts inferred from the emitted assembly by [llvm-mca](https://llvm.org/docs/CommandGuide/llvm-mca.html) are deterministic: they depend only on the code and LLVM's scheduling model for the chosen `-mcpu`, so one host can rank microarchitectures it does not own, and no CI runner noise enters. The public functions are `#[inline]`, so the crate's own assembly never contains them; a probe crate pins one function out of line. Run from the repository root with `llvm-mca` on `PATH` (Debian and Ubuntu install it under `/usr/lib/llvm-*/bin`).

```sh
d=$(mktemp -d) && mkdir "$d/src" && cat > "$d/Cargo.toml" <<EOF
[package]
name = "probe"
version = "0.0.0"
edition = "2021"
[lib]
crate-type = ["cdylib"]
[dependencies]
metallic = { path = "$PWD" }
[profile.release]
codegen-units = 1
lto = "fat"
EOF
echo '#[no_mangle] #[inline(never)] pub extern "C" fn probe(x: f64) -> f64 { metallic::exp(x) }' > "$d/src/lib.rs"
RUSTFLAGS="-Ctarget-cpu=x86-64-v3 --emit=asm" cargo build --release --manifest-path "$d/Cargo.toml"
awk '/^probe:/{p=1;next} p&&/^\s*\./{next} p{print} p&&/retq/{exit}' "$d/target/release/deps/probe.s" > fast.s
for cpu in haswell skylake znver2 znver3; do echo "$cpu $(llvm-mca -mcpu=$cpu -iterations=1000 fast.s | awk '/Total Cycles/{print $3/1000}')"; done
```

The cut keeps the straight-line code up to the first return, which for `exp` is the fast leg with every branch falling through; inspect `fast.s` before trusting it for a function whose first return is a special-case exit. Chaining 1000 iterations models a dependent call sequence (each call's input is the previous output through `xmm0`), so the figure is latency per call, not throughput. Change `-Ctarget-cpu` and `-mcpu` together to compare ISA levels: `x86-64-v2` emits no FMA, so its assembly differs, not only its model.

| Model (LLVM 15) | Cycles per dependent `exp` |
| --- | --- |
| haswell | 64 |
| skylake, icelake-server | 67 |
| znver2 | 69 |
| znver3 | 17 |
| Measured, Ryzen 9 7950X3D at ~4.4 GHz | ~57 |

The Intel and Zen 2 models land within 15% of the hardware. LLVM 15's Zen 3 model is incomplete and underestimates by 4×, and it is the closest model to the measured core: the estimate is only as good as the scheduling model, and AMD models have historically been the weaker ones. Zen 4 and Zen 5 models need LLVM 18 or newer. What the estimate cannot see: branch outcomes (the accurate-leg fallback rate must be measured and folded in as probability times accurate-leg cycles), cache misses (table loads are assumed L1 hits, which holds for these tables), and the random-input generation that the Criterion figures above include."""


def render(directory):
    api = public_api()
    defaults = api["f32_"] | api["f64_"]
    local = load_snapshot(directory / "local", "default", defaults)
    remote = load_snapshot(directory / "dl02", "default", defaults)
    quad = load_snapshot(directory / "dl02-f128", "f128", api["f128_"])
    snapshots = (local, remote, quad)
    heads = {output_of(data["source"]["git_head"]) for data in snapshots}
    require(len(heads) == 1, "Snapshots have different base commits")
    head = next(iter(heads))
    require(head.startswith("77e271f"), "Unexpected source base: " + head)
    require(len({data["source"]["cargo_lock_sha256"] for data in snapshots}) == 1,
            "Snapshots used different dependency locks")
    source_manifests = [{path: digest for path, digest in data["source"]["source_files_sha256"].items()
                         if not Path(path).name.startswith("._") and
                         (path in ("Cargo.toml", "Cargo.lock", "build.rs") or
                          path.startswith(("src/", "benches/", ".cargo/")))}
                        for data in snapshots]
    require(source_manifests[0] == source_manifests[1] == source_manifests[2],
            "Snapshots benchmark different library or harness source")
    for path, digest in source_manifests[0].items():
        require(hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest,
                "Current source differs from measured snapshot: " + path)
    setting_keys = ("warm_up_seconds", "measurement_seconds", "sample_size", "nresamples")
    require(len({tuple(data["settings"][key] for key in setting_keys) for data in snapshots}) == 1,
            "Snapshots use different Criterion settings")
    settings = local["settings"]
    canonical_hash = hashlib.sha256(json.dumps(source_manifests[0], sort_keys=True).encode()).hexdigest()
    source_trees = {data["source"]["source_tree_sha256"] for data in snapshots}
    source_tree_note = (" Canonical benchmark-source SHA-256 (Cargo manifests/lock, build script, "
                        "source, benchmark files, and any project Cargo config): `{}`.".format(canonical_hash))
    if len(source_trees) == 1:
        source_tree_note += (" Shared full source-tree SHA-256 (also including the runner): "
                             "`{}`.".format(next(iter(source_trees))))
    if any(Path(path).name.startswith("._") for data in snapshots
           for path in data["source"]["source_files_sha256"]):
        source_tree_note += (" Remote raw manifests additionally record macOS AppleDouble "
                             "`._*` resource-fork sidecars introduced during transfer. They "
                             "are not compiled and are excluded from the canonical source "
                             "comparison; the archived full manifests remain unchanged.")
    extras = sorted(target + "_" + band for target, bands in EXTRA_BANDS.items() for band in bands)
    tables = [("Local — f32", table(local, api["f32_"], "f32")),
              ("Local — f64", table(local, api["f64_"], "f64")),
              ("dl02.skymizer.com — f32", table(remote, api["f32_"], "f32")),
              ("dl02.skymizer.com — f64", table(remote, api["f64_"], "f64")),
              ("dl02.skymizer.com — binary128", table(quad, api["f128_"], "f128"))]
    gamma_tables = []
    for label, data in (("Local", local), ("dl02.skymizer.com", remote)):
        # Explicit extra-band labels contain no std aliases, so the mixed
        # f32/f64 rows can share one utility-comparator table.
        gamma_tables.append((label, table(data, extras, "f64", extra=True)))
    for data in snapshots:
        unused = set(data["measurements"]) - data["_consumed"]
        require(not unused, "Unreported benchmark lanes: " + ", ".join(sorted(unused)))
    probe = directory / "local" / "f128-probe.txt"
    require(probe.is_file(), "Missing recorded local binary128 compiler probe: " + str(probe))
    sections = [
        "# Benchmarks",
        "Measured {} (Asia/Taipei; UTC timestamps below) on the local machine and "
        "`dl02.skymizer.com`. The tables cover all "
        "**112 exposed math functions**: 46 f32 and 46 f64 functions on each host, plus all "
        "20 opt-in binary128 functions on dl02. Each default run also includes nine gamma "
        "band measurements, reported separately.".format(directory.name),
        "Times are Criterion **median nanoseconds per iteration**. **M/CORE = metallic / "
        "CORE-MATH** from the same benchmark run; below 1.00× favors metallic. Compare "
        "implementations within each row. Times characterize the recorded hosts, toolchains, "
        "and input distributions; they imply no universal ranking. A dash means that the harness "
        "has no corresponding comparison lane.",
        "## Method and provenance",
        "[The runner](tools/benchmark_snapshot.py) builds with `cargo bench --locked --no-run`, "
        "then executes one benchmark target at a time. Each lane uses {} seconds of warm-up, "
        "{} seconds of requested measurement time, {} samples, and {} bootstrap resamples. "
        "The archived median estimates and confidence intervals are preserved in each "
        "`results.json`; displayed ratios use the unrounded point estimates. Sampling noise "
        "and changing machine load still apply to close ratios.".format(
            settings["warm_up_seconds"], settings["measurement_seconds"],
            settings["sample_size"], settings["nresamples"]),
        "The timed iteration includes **fresh random-input generation and the function call**; "
        "the baseline lanes independently draw from the same distributions. RNG overhead "
        "can dominate inexpensive primitives such as FMA. These are sampled workloads, "
        "rather than isolated instruction latencies or exhaustive correctness tests.",
        "[Shared f32/f64 samplers](benches/bench.rs) use value-uniform draws for bounded "
        "ranges, representation-uniform draws for open-ended ranges, and random significands "
        "with uniformly selected exponents for `Exponents` / `PositiveExponents`. "
        "`Exponents` also randomizes the sign. Full-representation draws include subnormals, "
        "infinities, and NaNs. [Binary128 samplers](benches/bench128.rs) assemble the floating "
        "bits directly; [log1pq](benches/log1pq.rs) has its own sign-dependent exponent band. "
        "Every function name below links to its exact input ranges and lanes. Functions and "
        "precisions use different bands: some retain domain errors or early returns, while "
        "others concentrate on the active kernel. The powq base uses positive magnitudes "
        "so random negative noninteger powers do not dominate its timing.",
        "CORE-MATH is the principal comparison because it shares metallic's correct-rounding "
        "target. Standard-library, Rust `libm`, and GCC libquadmath lanes provide additional "
        "context; transcendental accuracy contracts differ. On dl02, most `f128::` methods "
        "call glibc; the `sqrtq` std lane resolves to Rust `compiler_builtins`, and the "
        "`exp10q` system lane is explicitly `glibc::exp10f128`. See "
        "[README baseline details](README.md#baselines). There is no CORE-MATH lane for "
        "`log2q`, `log10q`, `log1pq`, `powq`, `sinq`, `cosq`, or `tanq`; their libquadmath "
        "times remain separate from the CORE-MATH ratio. `compound` has no equivalent "
        "comparison entry point, so its row contains metallic alone.",
        "The local AArch64 default target enables hardware FMA. On dl02, both Rust's "
        "`RUSTFLAGS` and CORE-MATH's `TARGET_CPU` select `x86-64-v3`, which includes FMA.",
        "The measured source is based on commit `{}` with six added benchmark targets "
        "(`atan2f`, `erff`, `erfcf`, `fma`, `fmaf`, `compound`) and the powq positive-base "
        "sampler correction. All three snapshots have identical library/harness file hashes "
        "and dependency locks. The source manifest and working-tree state are archived in "
        "the results. Locked benchmark dependencies: {}.{}".format(
            head, dependencies(local["_directory"] / "Cargo.lock.snapshot"), source_tree_note),
        "## Hosts and run conditions",
        environment_table(snapshots),
        background_workloads(local, remote, quad),
        "**Local binary128: unavailable with the current benchmark dependencies.** "
        "The local Darwin C compiler rejects CORE-MATH's required `__float128` type "
        "([recorded compiler probe]({})). The benchmark runner therefore requires "
        "x86-64 GNU/Linux for these targets; dl02 supplies all 20 measurements. This "
        "limitation concerns the benchmark dependency build, not the availability of "
        "metallic's opt-in Rust library code.".format(link(probe)),
    ]
    for title, content in tables:
        sections.extend(("## " + title, content))
    sections.extend(("## Supplementary gamma bands",
                     "These nine workloads per host reuse three public functions over narrower input "
                     "bands. They are additional workloads and are excluded from the "
                     "112-function coverage count."))
    for title, content in gamma_tables:
        sections.extend(("### " + title, content))
    sections.extend(("## Reproducing the snapshots",
                     "Use the recorded Rust and C compiler versions, the same measured source, "
                     "and the archived lock file. The commands below use fresh output and "
                     "target directories: the runner refuses existing measurements so older "
                     "Criterion data cannot enter a new snapshot. Run the two dl02 commands "
                     "serially on `dl02.skymizer.com`; run the local command on the local host. "
                     "The f128 runner selects `+nightly --features f128` and needs `CC=clang`. "
                     "Preserve that nightly version when reproducing these measurements."))
    for label, data in (("local", local), ("dl02", remote), ("dl02-f128", quad)):
        sections.extend(("### " + label, reproduce(data, label)))
    sections.extend(("Regenerate this report from the committed snapshots:",
                     "```sh\npython3 tools/benchmark_report.py --snapshots {}\n```".format(link(directory)),
                     "[The report generator](tools/benchmark_report.py) validates 92/92/20 "
                     "public-function coverage, all nine supplementary bands in each default "
                     "run, archived lock hashes, matching source files, and consumption of every "
                     "recorded lane. Raw Criterion files and build/run logs remain in the "
                     "target directories recorded in each snapshot.",
                     STATIC_ESTIMATE))
    return "\n\n".join(sections) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--snapshots", type=Path, default=Path("benchmarks/2026-09-05"))
    parser.add_argument("--output", type=Path, default=Path("BENCHMARKS.md"))
    args = parser.parse_args()
    report = render(args.snapshots.resolve())
    args.output.write_text(report)
    print("Wrote {} (112 public functions; all measured lanes accounted for)".format(args.output))


if __name__ == "__main__":
    main()
