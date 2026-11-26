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


def unique_members(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f"duplicate output member: {key!r}")
        result[key] = value
    return result


def canonical(data):
    # Python integers preserve i64 exactly; comparing serialized values also
    # keeps booleans distinct from integers. The fixture has no floats.
    value = json.loads(
        data.decode("utf-8"), parse_float=reject_number,
        parse_constant=reject_number, object_pairs_hook=unique_members,
    )
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
    require(result.stdout.endswith(b"\n"), f"{label}: missing final newline")
    require(canonical(result.stdout) == expected, f"{label}: unexpected JSON {result.stdout!r}")
    return result.stdout


def expect_failure(result, diagnostic, label):
    require(
        result.returncode == 1 and not result.stdout and diagnostic in result.stderr,
        f"{label}: expected failure with empty stdout and {diagnostic!r}; got {result!r}",
    )


def json_inputs(data):
    Path("input.json").write_bytes(data)
    # A valid, unrelated stdin value makes wrong-channel reads observable for
    # both successful file exports and malformed/profile-excluded file cases.
    return ((["input.json"], b'{"stdin_only":true}'), (["json:", "-"], data))


def json_success(tool, data, expected, label, options=()):
    for args, stdin in json_inputs(data):
        export(tool, [*options, *args], canonical(expected), f"{label} {args}", data=stdin)


def json_values():
    # Independently authored answers, not outputs from either implementation.
    return [
        (b'null', b'null'), (b'true', b'true'), (b'false', b'false'),
        (b'0', b'0'), (b'-1', b'-1'), (b'[]', b'[]'), (b'{}', b'{}'),
        (b'-9223372036854775808', b'-9223372036854775808'),
        (b'9223372036854775807', b'9223372036854775807'),
        (b'9007199254740993', b'9007199254740993'),
        (b'"42"', b'"42"'), (b'"true"', b'"true"'),
        (b'"\\u0061\\uD83E\\uDD80"', '"a🦀"'.encode()),
        (b'"\\\"\\\\\\/\\b\\f\\n\\r\\t"', b'"\\\"\\\\/\\u0008\\u000c\\n\\r\\t"'),
        (b'"\\u0000"', b'"\\u0000"'),
        (b'"' + b'[' * 65 + b'}' * 65 + b'"', b'"' + b'[' * 65 + b'}' * 65 + b'"'),
        (b' [null,true,false,0,1,"0","true",[],{}] \r\n\t',
         b'[null,true,false,0,1,"0","true",[],{}]'),
        (b'{"":0,"a":1,"A":true,"\\u0062":"1","null":null}',
         b'{"":0,"a":1,"A":true,"b":"1","null":null}'),
        (b'{"x":{"a":1},"y":{"a":2}}', b'{"x":{"a":1},"y":{"a":2}}'),
    ]


def json_boundaries():
    # Count containers only: scalar=0, root object/array=1.
    cases = []
    for depth in (1, 63, 64):
        for left, right in ((b'[', b']'), (b'{"a":', b'}')):
            data = left * depth + b'0' + right * depth
            value = 0
            for _ in range(depth):
                value = [value] if left == b'[' else {"a": value}
            cases.append((data, json.dumps(value).encode()))
        data, value = b'0', 0
        for layer in range(depth):
            if layer % 2:
                data, value = b'{"a":' + data + b'}', {"a": value}
            else:
                data, value = b'[' + data + b']', [value]
        cases.append((data, json.dumps(value).encode()))
    limit = 1024 * 1024
    # Multibyte text hits the byte bound well before a character-count bound.
    text = b'"' + 'é'.encode() * ((limit - 2) // 2) + b'"'
    require(len(text) == limit, "incorrect UTF-8 boundary fixture")
    expected = json.dumps('é' * 524287, ensure_ascii=False).encode()
    cases.extend([(text, expected), (b'null' + b' ' * (limit - 4), b'null'),
                  (b'null' + b' ' * (limit - 5), b'null')])
    return cases


def json_oracle_gaps(cue):
    # Upstream accepts these excluded values. They are NOT parity passes.
    for data, expected in (
        (b'-0', b'0'), (b'9223372036854775808', b'9223372036854775808'),
        (b'{"a":1,"a":1}', b'{"a":1}'),
        (b'{"a":1,"\\u0061":1}', b'{"a":1}'),
    ):
        json_success(cue, data, expected, "Go-only excluded JSON")
    for args, stdin in json_inputs(b'1.5'):
        result = run(cue, ["export", *args], data=stdin)
        require(result.returncode == 0 and not result.stderr and result.stdout == b'1.5\n',
                f"Go-only float control: {result!r}")
    for args, stdin in json_inputs(b'{"a":1,"a":2}'):
        expect_failure(run(cue, ["export", *args], data=stdin),
                       b'conflicting values', "Go unequal duplicate gap")


def json_profile(rust, cue):
    for index, (data, expected) in enumerate(json_values() + json_boundaries()):
        for name, tool in (("Rust", rust), ("Go", cue)):
            json_success(tool, data, expected, f"{name} JSON value/boundary {index}")
    # Decoded key equality is not Unicode normalization. Go CUE conflates these
    # distinct keys: keep this upstream discrepancy outside parity passes.
    distinct_keys = '{"é":1,"é":2}'.encode()
    json_success(rust, distinct_keys, '{"é":2,"é":1}'.encode(), "distinct decoded keys")
    for args, stdin in json_inputs(distinct_keys):
        expect_failure(run(cue, ["export", *args], data=stdin),
                       b'conflicting values', "Go normalized-key gap")
    selected = b'{"answer":9223372036854775807,"items":[null,true,"1"],"int":false}'
    for option, expected in ((["-e", "answer"], b'9223372036854775807'),
                             (["--expression=items"], b'[null,true,"1"]'),
                             (["--expression", "int"], b'false')):
        for tool in (rust, cue):
            json_success(tool, selected, expected, "JSON selection", option)

    malformed = [
        b'', b' ', b'{', b'[1,]', b'{"a":}', b'{"a":1,}', b'{a:1}',
        b'1 2', b'null\n{}', b'{}{}', b'01', b'-01', b'+1', b'0x10',
        b'NaN', b'Infinity', b'undefined', b'//comment\n0', b'/*x*/0',
        b'"raw\nline"', b'"\\x41"', b'"\\u00xz"', b'1e', b'1.',
        b'x' + b'[' * 65,
        b'0 ' + b'[' * 65 + b'0' + b']' * 65,
    ]
    invalid_unicode = [
        b'"\xff"', b'"\xc0\xaf"', b'"\xe2\x82"', b'"\xed\xa0\x80"',
        b'"\\uD800"', b'"\\uDC00"', b'"\\uD800x"', b'"\\uD800\\u0041"',
        b'{"\\uD800":0}', b'"\x00"', b'\xef\xbb\xbfnull',
    ]
    excluded = [
        b'-0', b'1.0', b'-0.0', b'1e0', b'1E+2', b'1e9999',
        b'9223372036854775808', b'-9223372036854775809',
        b'{"a":1,"a":1}', b'{"a":1,"a":2}', b'{"a":1,"\\u0061":1}',
        b'{"outer":{"x":0,"x":0}}', b'{"":0,"":1}',
        b'[' * 65 + b'0' + b']' * 65,
        b'{"a":' * 65 + b'0' + b'}' * 65,
        b'[{"a":' * 32 + b'[' + b'0' + b']' + b'}]' * 32,
        b'null' + b' ' * (1024 * 1024 - 3),
        b'"' + 'é'.encode() * ((1024 * 1024 - 2) // 2) + b'" ',
    ]
    for category, values in ((b'invalid JSON:', malformed + invalid_unicode),
                             (b'unsupported JSON:', excluded)):
        for index, data in enumerate(values):
            for args, stdin in json_inputs(data):
                result = run(rust, ["export", *args], data=stdin)
                expect_failure(result, category, f"JSON {category!r} {index} {args}")
                require(result.stderr.startswith(b'cuestacean: ' + category),
                        f"wrong JSON diagnostic prefix: {result.stderr!r}")
    # Syntax errors observable upstream; Unicode repair behavior is not parity.
    for data in malformed:
        for args, stdin in json_inputs(data):
            expect_failure(run(cue, ["export", *args], data=stdin), b'invalid JSON',
                           f"Go malformed explicit JSON {args}")
    json_oracle_gaps(cue)

    for data, diagnostic in ((b'{}', b'record has no field named answer'),
                             (b'42', b'--expression requires a concrete record'),
                             (b'[]', b'--expression requires a concrete record')):
        for args, stdin in json_inputs(data):
            expect_failure(run(rust, ["export", "-e", "answer", *args], data=stdin),
                           diagnostic, "JSON selector failure")
    # Selection must not hide malformed or excluded unselected siblings.
    for data, category in (
        (b'{"answer":42,"other":}', b'invalid JSON:'),
        (b'{"answer":42,"other":1.0}', b'unsupported JSON:'),
        (b'{"answer":42,"other":{"x":0,"x":0}}', b'unsupported JSON:'),
    ):
        for args, stdin in json_inputs(data):
            expect_failure(run(rust, ["export", "-e", "answer", *args], data=stdin),
                           category, "whole JSON document before selector")
    require(not Path("missing.json").exists(), "missing-file fixture unexpectedly exists")
    expect_failure(run(rust, ["export", "missing.json"], data=b'{"answer":42}'),
                   b'cannot read missing.json', "missing JSON file cannot retry valid stdin")
    for selector in ('', 'true', 'false', 'null', 'a..b', '"a"', '_a', '#A', 'é', '1a'):
        expect_failure(run(rust, ["export", f"--expression={selector}", "json:", "-"],
                           data=b'{}'), b'--expression requires a dot-separated ASCII field path',
                       "unchanged JSON selector grammar")
    # Explicit JSON never retries as CUE. Conversely .cue/bare stdin never sniff JSON.
    for data in (b'40 + 2', b'answer:42', b'close({"a":1})'):
        for args, stdin in json_inputs(data):
            expect_failure(run(rust, ["export", *args], data=stdin), b'invalid JSON:',
                           "no JSON-to-CUE fallback")
    for args in (["json:"], ["json:", "-", "input.json"],
                 ["input.json", "input.cue"], ["yaml:", "-"], ["JSON:", "-"],
                 ["json:-"], ["input.JSON"]):
        expect_failure(run(rust, ["export", *args], data=b'0'),
                       b'cuestacean:', "only one explicit .json or literal json: -")
    Path("input.cue").write_bytes(b'{"answer":42}')
    for args, stdin in ((["input.cue"], b'42'), (["-"], b'{"answer":42}')):
        expect_failure(run(rust, ["export", *args], data=stdin),
                       b'expected signed decimal integer', "no CUE-to-JSON sniff/retry")


def nested_selection(rust, cue):
    source = (b'scope: close({"x":42,"end":null,"items":[1],'
              b'"answer":close({"case":7}),"true":8})\nother: 0\n')
    data = b'{"scope":{"x":42,"end":null,"items":[1],"answer":{"case":7},"true":8},"scope.x":99}'
    Path("nested.cue").write_bytes(source)
    routes = [(["nested.cue"], b"42"), (["-"], source), *json_inputs(data)]
    for args, stdin in routes:
        for options, expected in (
            (["-e", "scope.x"], b'42'),
            (["--expression", "scope.end"], b'null'),
            (["--expression=scope.answer.case"], b'7'),
            (["-e", "scope.true"], b'8'),
            (["-e", "scope.items"], b'[1]'),
        ):
            for tool in rust, cue:
                export(tool, [*options, *args], canonical(expected), f"nested {options} {args}", data=stdin)
        for path, rust_message, cue_message in (
            ("scope.missing", b"record scope has no field named missing", b"undefined field: missing"),
            ("scope.end.x", b"field scope.end is not a record", b"invalid operand scope.end"),
        ):
            for tool, diagnostic in ((rust, rust_message), (cue, cue_message)):
                expect_failure(run(tool, ["export", *args, "-e", path], data=stdin),
                               diagnostic, f"nested {path} {args}")
        expect_failure(run(rust, ["export", *args, "-e", "scope.items.x"], data=stdin),
                       b"field scope.items is not a record", "a list is not a record")
    # A literal dotted key is never a fallback for missing traversal segments.
    for args, stdin in json_inputs(b'{"scope":{},"scope.x":99}'):
        for tool, diagnostic in ((rust, b"record scope has no field named x"),
                                 (cue, b"undefined field: x")):
            expect_failure(run(tool, ["export", *args, "-e", "scope.x"], data=stdin),
                           diagnostic, "no literal dotted-key fallback")


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
        # Feed original stdout bytes directly, without normalization/adapters.
        for consumer, consumer_tool in (("Go", cue), ("Rust", rust)):
            export(consumer_tool, ["json:", "-"], expected,
                   f"{producer} -> {consumer} explicit JSON stdin", data=data)
        export(cue, ["-"], expected, f"{producer} -> Go bare stdin", data=data)
        expect_failure(run(rust, ["export", "-"], data=data),
                       b"source byte 0: expected signed decimal integer",
                       f"{producer} -> Rust bare stdin (retained gap)")

    json_profile(rust, cue)
    nested_selection(rust, cue)

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

    print("Explicit JSON round trips/profile, CUE parity, and retained compatibility gaps verified")


if __name__ == "__main__":
    main()
