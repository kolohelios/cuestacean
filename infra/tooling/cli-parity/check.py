"""Public CLI parity and explicit compatibility gaps, without input adapters."""

import json
import os
from pathlib import Path
import subprocess
import sys


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def reject_number(text):
    raise ValueError(f"unexpected non-integer JSON number: {text}")


def canonical(data):
    # Python integers preserve i64 exactly; comparing serialized values also
    # keeps booleans distinct from integers. The fixture has no floats.
    value = json.loads(data, parse_float=reject_number, parse_constant=reject_number)
    return json.dumps(value, sort_keys=True, ensure_ascii=False)


def run(tool, args, *, data=None):
    return subprocess.run(
        [tool, *args],
        input=data if data is not None else b"",
        capture_output=True,
        env={"HOME": str(Path("home").resolve()), "PATH": "/nonexistent"},
        timeout=30,
        check=False,
    )


def export(tool, args, expected, label, *, data=None):
    result = run(tool, ["export", *args], data=data)
    require(result.returncode == 0 and not result.stderr, f"{label}: {result!r}")
    require(canonical(result.stdout) == expected, f"{label}: unexpected JSON {result.stdout!r}")
    return result.stdout


def expect_failure(result, diagnostic, label):
    require(
        result.returncode == 1 and not result.stdout and diagnostic in result.stderr,
        f"{label}: expected failure with empty stdout and {diagnostic!r}; got {result!r}",
    )


def main():
    rust, cue = map(Path, sys.argv[1:])
    for tool in rust, cue:
        require(tool.is_absolute() and tool.is_file() and os.access(tool, os.R_OK | os.X_OK), tool)
    Path("home").mkdir()
    version = run(cue, ["version"])
    require(version.returncode == 0 and not version.stderr, f"cannot read CUE version: {version!r}")
    require(b"CUE language version v0.16.1" in version.stdout.splitlines(), "unexpected CUE pin")

    fixtures = Path(__file__).resolve().parent
    expected = canonical((fixtures / "portable.json").read_bytes())
    original = str(fixtures / "portable.cue")
    for producer, tool in (("Go", cue), ("Rust", rust)):
        data = export(tool, [original], expected, f"{producer} first export")
        for args, rust_diagnostic in (
            (["-"], b"source byte 0: expected signed decimal integer"),
            (["json:", "-"], b"export requires exactly one input"),
        ):
            export(cue, args, expected, f"{producer} -> Go {args}", data=data)
            # These are tracked rejections, not successful round trips.
            result = run(rust, ["export", *args], data=data)
            expect_failure(result, rust_diagnostic, f"{producer} -> Rust {args} (JSON input gap)")

    forward = str(fixtures / "defaults-forward.cue")
    reverse = str(fixtures / "defaults-reverse.cue")
    # The pinned evaluator can evaluate the forward expression but cannot
    # export it. The Rust parser does not support this syntax in either order.
    ordinary = run(cue, ["eval", forward])
    require(ordinary.returncode == 0 and ordinary.stdout and not ordinary.stderr, ordinary)
    expect_failure(run(cue, ["export", forward]), b"nested: incomplete value 13 | 17", "Go defaults forward")
    export(cue, [reverse], canonical(b'{"flat":2,"nested":13}'), "Go defaults reverse")
    for path in forward, reverse:
        expect_failure(run(rust, ["export", path]), b"expected signed decimal integer", f"Rust defaults gap: {path}")

    print("Complex export parity, Go round trips, and explicit Rust parity gaps verified")


if __name__ == "__main__":
    main()
