#!/usr/bin/env python3
"""Measure two explicit source CLIs; no builds or provider discovery."""

import argparse
import csv
import hashlib
import json
import math
import os
from pathlib import Path
import random
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
import platform

COLUMNS = ("track", "case", "engine", "metric", "unit", "iterations", "samples")
ENGINES = ("source-cli", "go-cue")


def reject_number(text):
    raise ValueError(f"unsupported JSON number: {text}")


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def read_json(text):
    value = json.loads(text, parse_float=reject_number, parse_constant=reject_number,
                       object_pairs_hook=unique_object)

    def check(node):
        if type(node) is int and not -(2**63) <= node < 2**63:
            raise ValueError("JSON integer outside signed i64")
        if isinstance(node, dict):
            for child in node.values():
                check(child)
        elif isinstance(node, list):
            for child in node:
                check(child)
    check(value)
    return value


def same_json(left, right):
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return left.keys() == right.keys() and all(
            same_json(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(map(same_json, left, right))
    return left == right


def load_cases(path):
    manifest = read_json(path.read_text(encoding="utf-8"))
    if (type(manifest) is not dict or set(manifest) != {"version", "cases"}
            or type(manifest["version"]) is not int or manifest["version"] != 1
            or type(manifest["cases"]) is not list or len(manifest["cases"]) != 12):
        raise ValueError("cases require version 1 and exactly twelve cases")
    seen = set()
    for case in manifest["cases"]:
        if (type(case) is not dict or set(case) != {"id", "source", "expected"}
                or type(case["id"]) is not str
                or not re.fullmatch(r"[a-z][a-z0-9-]{0,63}", case["id"])
                or case["id"] in seen or type(case["source"]) is not str
                or not case["source"]):
            raise ValueError("invalid or duplicate case")
        seen.add(case["id"])
    return manifest["cases"]


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def provider(path):
    resolved = path.resolve(strict=True)
    if not resolved.is_file() or not os.access(resolved, os.R_OK | os.X_OK):
        raise ValueError(f"not a readable executable file: {path}")
    return {"path": str(path), "resolved": str(resolved), "sha256": sha256(resolved)}


def invoke(argv, env, cwd, timeout, timed=False):
    start = time.perf_counter_ns() if timed else 0
    result = subprocess.run(argv, env=env, cwd=cwd, timeout=timeout,
                            capture_output=True, encoding="utf-8", check=False)
    elapsed = time.perf_counter_ns() - start if timed else None
    if result.returncode != 0 or result.stderr:
        raise ValueError(f"command failed {argv!r}: exit {result.returncode}; {result.stderr}")
    if timed and elapsed <= 0:
        raise ValueError("nonpositive elapsed nanoseconds")
    return result.stdout, elapsed


def check_output(text, expected):
    if not same_json(read_json(text), expected):
        raise ValueError("output differs from independent expected JSON")


def prepare(cases, directory, source, cue):
    commands = {}
    for case in cases:
        path = directory / (case["id"] + ".cue")
        path.write_text(case["source"], encoding="utf-8")
        commands[(case["id"], "source-cli")] = [source, str(path)]
        commands[(case["id"], "go-cue")] = [cue, "export", "--out", "json", str(path)]
    return commands


def validate_all(cases, commands, env, cwd, timeout):
    for case in cases:
        for engine in ENGINES:
            try:
                text, _ = invoke(commands[(case["id"], engine)], env, cwd, timeout)
                check_output(text, case["expected"])
            except (ValueError, OSError, subprocess.SubprocessError) as error:
                raise ValueError(f"preflight {case['id']}/{engine}: {error}") from error


def collect(cases, commands, args, env, cwd):
    # ALL cases/engines must validate before even the first recorded clock.
    validate_all(cases, commands, env, cwd, args.timeout)
    expected = {case["id"]: case["expected"] for case in cases}
    series = {(case["id"], engine, metric): [] for metric in ("latency", "peak-rss")
              for case in cases for engine in ENGINES}
    rng = random.Random(args.seed)
    schedule = []
    rss_file = cwd / "rss.txt"
    for stage, rounds in (("warmup", args.warmups), ("latency", args.samples),
                          ("peak-rss", args.rss_samples)):
        for round_index in range(rounds):
            order = list(commands)
            rng.shuffle(order)
            schedule.append({"stage": stage, "round": round_index,
                             "order": [f"{case}/{engine}" for case, engine in order]})
            for case, engine in order:
                try:
                    argv = commands[(case, engine)]
                    if stage == "peak-rss":
                        rss_file.unlink(missing_ok=True)
                        argv = [str(args.time), "-f", "%M", "-o", str(rss_file), "--", *argv]
                    text, elapsed = invoke(argv, env, cwd, args.timeout, stage == "latency")
                    check_output(text, expected[case])  # Outside the latency interval.
                    if stage == "peak-rss":
                        raw = rss_file.read_text(encoding="ascii")
                        if not re.fullmatch(r"[0-9]+\n?", raw):
                            raise ValueError("invalid GNU time peak RSS")
                        series[(case, engine, stage)].append(int(raw))
                    elif stage == "latency":
                        series[(case, engine, stage)].append(elapsed)
                except (ValueError, OSError, subprocess.SubprocessError) as error:
                    raise ValueError(f"{stage} round {round_index} {case}/{engine}: {error}") from error
    return series, schedule


def summary(values):
    q1, _, q3 = (statistics.quantiles(values, n=4, method="inclusive")
                 if len(values) > 1 else [values[0]] * 3)
    return {"count": len(values), "median": statistics.median(values),
            "q1": q1, "q3": q3, "iqr": q3 - q1, "min": min(values), "max": max(values)}


def write_tsv(path, series):
    with path.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.writer(stream, delimiter="\t", lineterminator="\n")
        writer.writerow(COLUMNS)
        for (case, engine, metric), values in series.items():
            writer.writerow(("runtime", case, engine, metric,
                             "ns" if metric == "latency" else "KiB", 1,
                             ",".join(map(str, values))))


def optional_text(path):
    try:
        return Path(path).read_text(encoding="utf-8").strip()
    except OSError:
        return None


def host():
    cpuinfo = optional_text("/proc/cpuinfo") or ""
    model = next((line.split(":", 1)[1].strip() for line in cpuinfo.splitlines()
                  if line.startswith("model name")), None)
    memory = optional_text("/proc/meminfo") or ""
    total = next((line.split(":", 1)[1].strip() for line in memory.splitlines()
                  if line.startswith("MemTotal:")), None)
    affinity = sorted(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None
    cpu = affinity[0] if affinity else 0
    return {"uname": platform.uname()._asdict(), "cpu_model": model,
            "cpu_count": os.cpu_count(), "memory_total": total, "affinity": affinity,
            "governor_cpu": cpu,
            "governor": optional_text(f"/sys/devices/system/cpu/cpu{cpu}/cpufreq/scaling_governor"),
            "load": list(os.getloadavg())}


def publish(directory, series, metadata):
    # Exclusive creation also rejects a competing invocation after initial checks.
    directory.mkdir(mode=0o700)
    try:
        write_tsv(directory / "raw.tsv", series)
        metadata["raw_tsv_sha256"] = sha256(directory / "raw.tsv")
        metadata["complete"] = True
        pending = directory / ".metadata.json"
        pending.write_text(json.dumps(metadata, indent=2, allow_nan=False) + "\n", encoding="utf-8")
        pending.replace(directory / "metadata.json")  # Completion is published LAST.
    except BaseException:
        shutil.rmtree(directory)
        raise


def absolute(text):
    path = Path(text)
    if not path.is_absolute():
        raise argparse.ArgumentTypeError("requires an absolute path")
    return path


def revision(text):
    if not re.fullmatch(r"[a-fA-F0-9]{40}", text):
        raise argparse.ArgumentTypeError("requires exactly 40 hexadecimal digits")
    return text.lower()


def bounded(low, high):
    def parse(text):
        if not re.fullmatch(r"[0-9]+", text) or not low <= int(text) <= high:
            raise argparse.ArgumentTypeError(f"requires integer {low}..{high}")
        return int(text)
    return parse


def timeout_seconds(text):
    try:
        value = float(text)
    except ValueError as error:
        raise argparse.ArgumentTypeError("invalid timeout") from error
    if not math.isfinite(value) or not 0 < value <= 3600:
        raise argparse.ArgumentTypeError("timeout must be finite, positive and <=3600")
    return value


def arguments(argv=None):
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    for name in ("source-cli", "cue", "time", "cases", "output"):
        parser.add_argument("--" + name, required=True, type=absolute, help="absolute path")
    for name in ("revision", "harness-revision"):
        parser.add_argument("--" + name, required=True, type=revision, help="declared full source revision")
    for name, default, low, high in (("samples", 31, 2, 1000), ("warmups", 5, 1, 1000),
                                     ("rss-samples", 5, 1, 1000), ("seed", 1729, 0, 2**32 - 1)):
        parser.add_argument("--" + name, default=default, type=bounded(low, high), help="count/seed")
    parser.add_argument("--timeout", default=30, type=timeout_seconds, help="child timeout seconds")
    return parser.parse_args(argv)


def run(args):
    if sys.platform != "linux":
        raise ValueError("Linux required for GNU time RSS KiB")
    if args.output.exists() or args.output.is_symlink() or not args.output.parent.is_dir():
        raise ValueError("output must be new with an existing parent directory")
    tools = {name: provider(getattr(args, name.replace("-", "_")))
             for name in ("source-cli", "cue", "time")}
    script = Path(__file__).resolve()
    identities = {"script": {"path": str(script), "sha256": sha256(script)},
                  "cases": {"path": str(args.cases), "sha256": sha256(args.cases)}}
    cases = load_cases(args.cases)
    started = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    before = host()
    with tempfile.TemporaryDirectory(prefix="cuestacean-performance-") as temporary:
        cwd = Path(temporary)
        env = {"PATH": "/nonexistent", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "TZ": "UTC"}
        for name in ("HOME", "XDG_CONFIG_HOME", "XDG_CACHE_HOME", "TMPDIR"):
            path = cwd / name.lower()
            path.mkdir()
            env[name] = str(path)
        versions = {"source-cli": None, "python": sys.version}
        for name, flag in (("cue", "version"), ("time", "--version")):
            versions[name], _ = invoke([tools[name]["resolved"], flag], env, cwd, args.timeout)
        if "gnu time" not in versions["time"].lower():
            raise ValueError("time provider is not GNU time")
        # Execute resolved, hash-checked files, never another PATH candidate.
        args.time = Path(tools["time"]["resolved"])
        commands = prepare(cases, cwd, tools["source-cli"]["resolved"], tools["cue"]["resolved"])
        series, schedule = collect(cases, commands, args, env, cwd)
    for identity in [*tools.values(), *identities.values()]:
        if sha256(Path(identity.get("resolved", identity["path"]))) != identity["sha256"]:
            raise ValueError("provider/script/cases identity changed during run")
    metadata = {"version": 1, "revision": args.revision, "harness_revision": args.harness_revision,
                "argv": sys.argv, "settings": {key: str(value) if isinstance(value, Path) else value
                                               for key, value in vars(args).items()},
                "started_utc": started, "finished_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                "run_id": f"{started}/{args.seed}", "tools": tools, "inputs": identities,
                "versions": versions, "python": provider(Path(sys.executable)),
                "environment": env, "host_before": before, "host_after": host(),
                "clock": vars(time.get_clock_info("perf_counter")), "schedule": schedule,
                "validated_cases": [case["id"] for case in cases],
                "summaries": [{"case": case, "engine": engine, "metric": metric, **summary(values)}
                              for (case, engine, metric), values in series.items()]}
    publish(args.output, series, metadata)


def main(argv=None):
    args = arguments(argv)
    try:
        run(args)
    except (ValueError, OSError, subprocess.SubprocessError, KeyboardInterrupt) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
