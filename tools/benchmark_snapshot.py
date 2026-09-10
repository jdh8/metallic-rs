#!/usr/bin/env python3
"""Build all math benches, run them serially, and preserve a benchmark snapshot.

Example (set platform-appropriate Rust and C target flags in the environment):
    python3 tools/benchmark_snapshot.py --precision default \
        --target-dir target/snapshot-local --output benchmarks/2026-09-05/local

The output directory contains compact results.json and Cargo.lock.snapshot.
Build/run logs, the source diff, and Criterion's original JSON stay in target-dir.
An existing Criterion result directory is refused, so stale measurements cannot
enter a snapshot. --build-only may precede a full invocation on the same paths.
Requires Python 3.9+, Cargo, and the toolchain selected by the precision.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shlex
import shutil
import socket
import subprocess
import sys
import time


ROOT = Path(__file__).resolve().parent.parent


def utc_now():
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def probe(command, env=None):
    """A missing optional metadata utility is recorded, never installed."""
    try:
        result = subprocess.run(
            command, cwd=ROOT, env=env, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, text=True, timeout=15, check=False,
        )
        return {"command": command, "exit_code": result.returncode,
                "output": result.stdout.strip()}
    except (OSError, subprocess.TimeoutExpired) as error:
        return {"command": command, "error": str(error)}


def machine_state():
    state = {"utc": utc_now(), "load_average": list(os.getloadavg())}
    if sys.platform == "darwin":
        state["memory"] = probe(["vm_stat"])
        state["swap"] = probe(["sysctl", "vm.swapusage"])
        ps = probe(["ps", "-A", "-o", "pid=,pcpu=,pmem=,rss=,comm="])
    elif sys.platform.startswith("linux"):
        keys = {"MemTotal", "MemFree", "MemAvailable", "Buffers", "Cached",
                "SwapTotal", "SwapFree", "Dirty"}
        state["memory_kib"] = {
            line.split(":")[0]: int(line.split()[1])
            for line in Path("/proc/meminfo").read_text().splitlines()
            if line.split(":")[0] in keys
        }
        for resource in ("cpu", "memory", "io"):
            path = Path("/proc/pressure") / resource
            if path.exists():
                state[resource + "_pressure"] = path.read_text().strip()
        ps = probe(["ps", "-eo", "pid=,pcpu=,pmem=,rss=,comm="])
    else:
        return state
    # Record executable names only: command arguments may contain private data.
    rows = []
    for line in ps.get("output", "").splitlines():
        fields = line.split(None, 4)
        try:
            rows.append({"pid": int(fields[0]), "cpu_percent": float(fields[1]),
                         "memory_percent": float(fields[2]),
                         "rss_kib": int(fields[3]), "executable": fields[4]})
        except (ValueError, IndexError):
            continue
    state["top_cpu"] = sorted(rows, key=lambda row: -row["cpu_percent"])[:5]
    state["top_memory"] = sorted(rows, key=lambda row: -row["rss_kib"])[:5]
    return state


def machine_metadata(toolchain, env):
    data = {
        "hostname": socket.gethostname(), "platform": platform.platform(),
        "machine": platform.machine(), "logical_cpus": os.cpu_count(),
        "python": sys.version, "rustc": probe(["rustc"] + toolchain + ["-vV"], env),
        "cargo": probe(["cargo"] + toolchain + ["-V"], env),
        "c_compiler": probe(shlex.split(env.get("CC", "cc")) + ["--version"], env),
    }
    if sys.platform == "darwin":
        data["hardware"] = probe(["sysctl", "machdep.cpu.brand_string", "hw.model",
                                  "hw.physicalcpu", "hw.logicalcpu", "hw.memsize"])
        data["os_version"] = probe(["sw_vers"])
        data["power"] = probe(["pmset", "-g", "batt"])
    elif sys.platform.startswith("linux"):
        data["hardware"] = probe(["lscpu"])
        data["libc"] = probe(["ldd", "--version"])
        cpu = Path("/sys/devices/system/cpu/cpu0/cpufreq")
        data["frequency_policy"] = {
            name: (cpu / name).read_text().strip()
            for name in ("scaling_driver", "scaling_governor", "scaling_min_freq",
                         "scaling_max_freq", "energy_performance_preference")
            if (cpu / name).exists()
        }
    return data


def source_metadata(raw_dir):
    diff = subprocess.run(["git", "diff", "--binary", "HEAD"], cwd=ROOT,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if diff.returncode == 0:
        (raw_dir / "source.patch").write_bytes(diff.stdout)
    paths = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "build.rs",
             Path(__file__).resolve()]
    paths.extend((ROOT / ".cargo").glob("*.toml"))
    for directory in ("src", "benches"):
        paths.extend(path for path in (ROOT / directory).rglob("*") if path.is_file())
    manifest = {str(path.relative_to(ROOT)): sha256(path.read_bytes())
                for path in sorted(set(paths))}
    return {
        "git_head": probe(["git", "rev-parse", "HEAD"]),
        "git_status": probe(["git", "status", "--porcelain"]),
        "tracked_diff_sha256": sha256(diff.stdout) if diff.returncode == 0 else None,
        "source_files_sha256": manifest,
        "source_tree_sha256": sha256(json.dumps(manifest, sort_keys=True).encode()),
        "cargo_lock_sha256": manifest["Cargo.lock"],
    }


def public_functions(precision):
    groups = ("f128_",) if precision == "f128" else ("f32_", "f64_")
    pattern = r"pub use (f32_|f64_|f128_)(?:::\w+)*::\{([^}]+)\};"
    names = []
    for group, body in re.findall(pattern, (ROOT / "src/lib.rs").read_text()):
        if group in groups:
            names.extend(name.strip() for name in body.split(",") if name.strip())
    if not names or any(not re.fullmatch(r"\w+", name) for name in names):
        raise RuntimeError("Could not identify the public math functions in src/lib.rs")
    return sorted(names)


def write_results(output, data):
    temporary = output / "results.json.tmp"
    temporary.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
    temporary.replace(output / "results.json")


def run_logged(command, log_path, env, data, phase):
    record = {"phase": phase, "command": command, "start_utc": utc_now(),
              "log": str(log_path)}
    data["commands"].append(record)
    state = machine_state()
    state["phase"] = phase
    data["machine_samples"].append(state)
    started = time.monotonic()
    print("[{}] {}".format(record["start_utc"], phase), flush=True)
    with log_path.open("w") as log:
        process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=log,
                                   stderr=subprocess.STDOUT)
        try:
            while True:
                try:
                    code = process.wait(timeout=60)
                    break
                except subprocess.TimeoutExpired:
                    state = machine_state()
                    state["phase"] = phase
                    data["machine_samples"].append(state)
                    print("[{}] {} still running".format(state["utc"], phase), flush=True)
        except BaseException:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
            raise
    record.update(end_utc=utc_now(), elapsed_seconds=time.monotonic() - started,
                  exit_code=code)
    if code:
        tail = "\n".join(log_path.read_text(errors="replace").splitlines()[-30:])
        raise RuntimeError("{} failed ({}):\n{}".format(phase, code, tail))
    return record


def collect_results(criterion_dir):
    results = {}
    for path in sorted(criterion_dir.glob("**/new/benchmark.json")):
        benchmark = json.loads(path.read_text())
        full_id = benchmark["full_id"]
        if full_id in results:
            raise RuntimeError("Duplicate Criterion full_id: " + full_id)
        estimates = json.loads((path.parent / "estimates.json").read_text())
        sample = json.loads((path.parent / "sample.json").read_text())
        results[full_id] = {
            "benchmark": benchmark,
            "estimates": {key: estimates[key] for key in ("mean", "median")},
            "sample_count": len(sample["times"]),
            "raw_directory": str(path.parent),
        }
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--target-dir", required=True, type=Path)
    parser.add_argument("--precision", choices=("default", "f128"), default="default")
    parser.add_argument("--build-only", action="store_true")
    parser.add_argument("--cpu", type=int, help="Linux CPU affinity for benchmark runs only")
    parser.add_argument("--warm-up-time", type=float, default=1.0)
    parser.add_argument("--measurement-time", type=float, default=2.0)
    parser.add_argument("--sample-size", type=int, default=100)
    parser.add_argument("--nresamples", type=int, default=10000)
    args = parser.parse_args()
    if args.warm_up_time <= 0 or args.measurement_time <= 0 or args.sample_size < 10:
        parser.error("Timing durations must be positive and sample-size at least 10")
    if args.nresamples <= 0:
        parser.error("nresamples must be positive")
    if args.cpu is not None and (not sys.platform.startswith("linux") or not shutil.which("taskset")):
        parser.error("--cpu requires Linux and taskset")
    if args.precision == "f128" and not (
        sys.platform.startswith("linux") and platform.machine() == "x86_64"
    ):
        parser.error("This repository's binary128 benches require x86-64 GNU/Linux")

    output, target = args.output.resolve(), args.target_dir.resolve()
    criterion_dir = target / "criterion"
    if criterion_dir.exists() and any(criterion_dir.iterdir()):
        parser.error("Existing Criterion data: choose a fresh --target-dir")
    existing = output / "results.json"
    if existing.exists() and json.loads(existing.read_text()).get("status") != "built":
        parser.error("Existing snapshot: choose a fresh --output")
    if not (ROOT / "Cargo.lock").is_file():
        parser.error("Cargo.lock is required for a locked, reproducible snapshot")
    output.mkdir(parents=True, exist_ok=True)
    raw_dir = target / "benchmark-snapshot"
    raw_dir.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target)
    env["CRITERION_HOME"] = str(criterion_dir)
    toolchain = ["+nightly"] if args.precision == "f128" else []
    if args.precision == "f128":
        env.setdefault("CC", "clang")
    cargo = ["cargo"] + toolchain
    metadata_command = cargo + ["metadata", "--locked", "--no-deps", "--format-version=1"]
    metadata_result = probe(metadata_command, env)
    if metadata_result.get("exit_code") != 0:
        raise RuntimeError("cargo metadata failed: " + str(metadata_result))
    metadata = json.loads(metadata_result["output"])
    package = next(item for item in metadata["packages"]
                   if Path(item["manifest_path"]).resolve() == ROOT / "Cargo.toml")
    targets = sorted(item["name"] for item in package["targets"]
                     if "bench" in item["kind"] and
                     ("f128" in item.get("required-features", [])) == (args.precision == "f128"))
    expected = public_functions(args.precision)
    missing = sorted(set(expected) - set(targets))
    if missing:
        raise RuntimeError("Public functions without benchmark targets: " + ", ".join(missing))

    interesting_env = {
        key: value for key, value in env.items()
        if key in {"RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CC", "CXX", "CFLAGS",
                   "CXXFLAGS", "CPPFLAGS", "LDFLAGS", "TARGET_CPU", "CARGO_TARGET_DIR",
                   "CRITERION_HOME", "CARGO_BUILD_JOBS", "CARGO_INCREMENTAL"}
        or key.startswith("CARGO_PROFILE_")
    }
    data = {
        "schema_version": 1, "status": "running", "start_utc": utc_now(),
        "precision": args.precision, "environment": interesting_env,
        "settings": {"warm_up_seconds": args.warm_up_time,
                     "measurement_seconds": args.measurement_time,
                     "sample_size": args.sample_size, "nresamples": args.nresamples,
                     "cpu_affinity": args.cpu},
        "targets": targets, "expected_public_functions": expected,
        "source": source_metadata(raw_dir), "machine": machine_metadata(toolchain, env),
        "machine_start": machine_state(), "machine_samples": [], "commands": [],
        "measurements": {},
    }
    shutil.copyfile(ROOT / "Cargo.lock", output / "Cargo.lock.snapshot")
    write_results(output, data)
    try:
        build = cargo + ["bench", "--locked", "--no-run", "--message-format=json"]
        if args.precision == "f128":
            build += ["--features", "f128"]
        for name in targets:
            build += ["--bench", name]
        build_log = raw_dir / "build.log"
        run_logged(build, build_log, env, data, "build")
        executables = {}
        for line in build_log.read_text(errors="replace").splitlines():
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                continue
            if (message.get("reason") == "compiler-artifact" and message.get("executable")
                    and "bench" in message["target"]["kind"]):
                executables[message["target"]["name"]] = message["executable"]
        if set(executables) != set(targets):
            raise RuntimeError("Build did not produce every selected bench executable")
        data["executables"] = executables
        if args.build_only:
            data["status"] = "built"
        else:
            data["measurement_start_utc"] = utc_now()
            data["machine_before_measurement"] = machine_state()
            options = ["--bench", "--warm-up-time", str(args.warm_up_time),
                       "--measurement-time", str(args.measurement_time),
                       "--sample-size", str(args.sample_size),
                       "--nresamples", str(args.nresamples), "--noplot"]
            ownership = {}
            for name in targets:
                command = [executables[name]] + options
                if args.cpu is not None:
                    command = ["taskset", "--cpu-list", str(args.cpu)] + command
                before = set(data["measurements"])
                run_logged(command, raw_dir / (name + ".log"), env, data, name)
                data["measurements"] = collect_results(criterion_dir)
                added = set(data["measurements"]) - before
                if "metallic::" + name not in added:
                    raise RuntimeError("Missing new metallic result for target " + name)
                for full_id in added:
                    data["measurements"][full_id]["target"] = name
                # Restore ownership recorded for previously completed targets.
                for full_id in before:
                    data["measurements"][full_id]["target"] = ownership[full_id]
                ownership = {key: value["target"] for key, value in data["measurements"].items()}
                write_results(output, data)
            missing = [name for name in expected if "metallic::" + name not in data["measurements"]]
            if missing:
                raise RuntimeError("Missing public function measurements: " + ", ".join(missing))
            data["coverage"] = {"public_functions": len(expected),
                                "benchmark_targets": len(targets),
                                "measurement_lanes": len(data["measurements"]), "missing": missing}
            data["status"] = "complete"
    except BaseException as error:
        data["status"] = "failed"
        data["error"] = str(error)
        raise
    finally:
        data["end_utc"] = utc_now()
        data["machine_end"] = machine_state()
        write_results(output, data)
    print("{}: {}".format(data["status"], output / "results.json"), flush=True)


if __name__ == "__main__":
    main()
