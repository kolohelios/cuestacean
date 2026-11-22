"""Fixed three-target warm-dependency recipe; see docs/performance-baseline.md.

Usage: PINNED_PYTHON bootstrap.py ABS_NIX ABS_PACKAGES_JSON NEW_ABS_DIR HARNESS_REV
The packages file is the coordinator's immutable, once-captured flake package map.
"""

import hashlib
import json
import os
from pathlib import Path
import random
import re
import subprocess
import sys
import time

NAMES = ("exports", "native-instruction-operand-cases", "native-codegen-core")
HEADER = "track\tcase\tengine\tmetric\tunit\titerations\tsamples\n"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    nix, capture, destination, revision = sys.argv[1:]
    paths = [Path(value) for value in (nix, capture, destination)]
    if not all(path.is_absolute() for path in paths):
        raise ValueError("all paths must be absolute")
    executable = paths[0].resolve(strict=True)
    if not executable.is_file() or not os.access(executable, os.R_OK | os.X_OK):
        raise ValueError("Nix must resolve to a readable executable regular file")
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("harness revision must be full lowercase hexadecimal")
    frozen = {str(path): sha(path) for path in (paths[1], executable, Path(__file__))}
    packages = json.loads(paths[1].read_text())
    roots = {name: Path(packages[name]) for name in NAMES}
    for root in roots.values():
        if not root.is_absolute() or root.suffix != ".drv" or not root.is_file():
            raise ValueError(f"invalid captured derivation: {root}")
    output = paths[2]
    output.mkdir()  # Exclusive ownership; never overwrite or resume another run.
    receipts = []

    def command(arguments, timed=False):
        argv = [str(executable), *arguments]
        start = time.perf_counter_ns()
        result = subprocess.run(argv, capture_output=True, timeout=1800, check=False)
        elapsed = time.perf_counter_ns() - start
        receipt = {"argv": argv, "status": result.returncode,
                   "stdout": result.stdout.decode(), "stderr": result.stderr.decode()}
        receipts.append(receipt)
        (output / "commands.json").write_text(json.dumps(receipts, indent=2) + "\n")
        if result.returncode:
            raise RuntimeError(f"Nix failed: {argv}; see {output / 'commands.json'}")
        return result.stdout, elapsed if timed else None

    def build(root, rebuild=False, timed=False):
        arguments = ["build", "--offline", "--no-link", "--json", "--option",
                     "sandbox", "true", "--option", "substitute", "false"]
        if rebuild:
            arguments.append("--rebuild")
        raw, elapsed = command([*arguments, f"{root}^*"], timed)
        result = json.loads(raw)
        if len(result) != 1 or set(result[0]["outputs"]) != {"out"}:
            raise ValueError("expected exactly one out output")
        return result[0]["outputs"]["out"], elapsed

    def identity(name):
        root = roots[name]
        target, _ = build(root)  # Readiness/identity is deliberately untimed.
        raw, _ = command(["path-info", "--json", "--json-format", "1", target])
        info = json.loads(raw)[target]
        if info["deriver"] != str(root):
            raise ValueError(f"output deriver mismatch: {name}")
        raw, _ = command(["derivation", "show", str(root)])
        return {"drv": str(root), "drv_sha256": sha(root), "output": target,
                "narHash": info["narHash"], "narSize": info["narSize"],
                "derivation": json.loads(raw)}

    before = {name: identity(name) for name in NAMES}
    samples = {(name, mode): [] for mode in ("cached", "rebuild") for name in NAMES}
    schedule, generator = [], random.Random(1729)
    load_before = os.getloadavg()
    for mode, count in (("cached", 7), ("rebuild", 3)):
        for _ in range(count):
            order = list(NAMES)
            generator.shuffle(order)
            schedule.append({"mode": mode, "order": order})
            for name in order:
                target, elapsed = build(roots[name], mode == "rebuild", True)
                if target != before[name]["output"] or elapsed <= 0:
                    raise ValueError(f"output/clock drift: {name}")
                samples[name, mode].append(elapsed)
    load_after = os.getloadavg()
    after = {name: identity(name) for name in NAMES}
    if before != after or any(sha(path) != digest for path, digest in frozen.items()):
        raise ValueError("derivation/output/NAR identity drift")
    lines = [HEADER]
    for (name, mode), values in samples.items():
        lines.append(f"bootstrap\t{name}\tnix\t{mode}\tns\t1\t"
                     + ",".join(map(str, values)) + "\n")
    (output / "raw.tsv").write_text("".join(lines))
    metadata = {"version": 1, "complete": True, "harness_revision": revision,
                "baseline_revision": "0148784521f1f1e5e51cfdcaa979dae6721c594d",
                "nix": str(executable), "nix_sha256": sha(executable),
                "capture_sha256": sha(capture), "script_sha256": sha(__file__),
                "roots": before, "schedule": schedule, "load_before": load_before,
                "load_after": load_after, "raw_sha256": sha(output / "raw.tsv")}
    (output / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, IndexError, TypeError, RuntimeError,
            subprocess.SubprocessError) as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(2)
