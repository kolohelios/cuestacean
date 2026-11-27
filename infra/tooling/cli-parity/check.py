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
                           data=b'{}'), b'--expression requires an ASCII field path',
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


def indexed_selection(rust, cue):
    # Independent concrete CUE and JSON inputs; never translate one into the other.
    source = (b'items:[close({"answer":[[42]],"true":7}),null,true]\n'
              b'empty:[]\nscalar:false\nobject:close({"x":1})\n')
    data = (b'{"items":[{"answer":[[42]],"true":7},null,true],'
            b'"empty":[],"scalar":false,"object":{"x":1},"items[3]":99}')
    Path("indexed.cue").write_bytes(source)
    routes = [(["indexed.cue"], b"0"), (["-"], source), *json_inputs(data)]
    for args, stdin in routes:
        for options, expected in (
            (["-e", "items[0].answer[0][0]"], b'42'),
            (["--expression=items[1]"], b'null'),
            (["--expression", "items[2]"], b'true'),
            (["-e", "items[0].true"], b'7'),
            (["-e", "items[0].answer[0]"], b'[42]'),
            (["-e", "items[0]"], b'{"answer":[[42]],"true":7}'),
        ):
            for tool in rust, cue:
                export(tool, [*options, *args], canonical(expected),
                       f"indexed {options} {args}", data=stdin)
        for path, rust_message, cue_message in (
            ("items[3]", b"index out of range [3] with length 3", b"index out of range [3] with length 3"),
            ("items[99]", b"index out of range [99] with length 3", b"index out of range [99] with length 3"),
            ("items[268435454]", b"index out of range [268435454] with length 3",
             b"index out of range [268435454] with length 3"),
            ("empty[0]", b"index out of range [0] with length 0", b"index out of range [0] with length 0"),
            ("scalar[0]", b"field scalar is not a list", b"invalid operand scalar"),
            ("items[1][0]", b"field items[1] is not a list", b"invalid operand items[1]"),
            ("items[2][0]", b"field items[2] is not a list", b"invalid operand items[2]"),
            ("items[0].missing", b"record items[0] has no field named missing", b"undefined field: missing"),
            ("items[1].x", b"field items[1] is not a record", b"invalid operand items[1]"),
        ):
            for tool, diagnostic in ((rust, rust_message), (cue, cue_message)):
                expect_failure(run(tool, ["export", *args, "-e", path], data=stdin),
                               diagnostic, f"indexed failure {path} {args}")
        # Go permits integer struct labels; our profile admits arrays only.
        expect_failure(run(rust, ["export", *args, "-e", "object[0]"], data=stdin),
                       b"field object is not a list", "record is not an array")

    # Syntax exclusions are not differential failures: some work upstream.
    unsupported = ('items[-1]', 'items[01]', 'items[0x0]', 'items[1+1]',
                   '[0]', 'items[0:1]', 'items[:1]', 'items["0"]',
                   'items[]', 'items[ 0]', 'items[0 ]', 'items[0',
                   'items[0]x', 'items.[0]', 'items[0].', 'items[0]._x')
    for args in (["missing.cue"], ["-"], ["missing.json"], ["json:", "-"]):
        for path in unsupported:
            result = run(rust, ["export", *args, f"--expression={path}"], data=b'!')
            expect_failure(result, b"--expression requires an ASCII field path",
                           f"invalid selector before input {path} {args}")
            require(result.stderr.startswith(b"cuestacean: --expression requires an ASCII field path"),
                    f"invalid selector diagnostic prefix: {result.stderr!r}")
        for index in ('268435455', '9223372036854775807', '18446744073709551616'):
            expect_failure(run(rust, ["export", *args, "-e", f"items[{index}]"], data=b'!'),
                           b"unsupported selector index:", "oversized selector before input")

    # Neither dotted nor bracketed literal keys can rescue failed traversal.
    no_fallback = (
        (b'close({"items":[],"items[0]":99})', b'{"items":[],"items[0]":99}',
         'items[0]', b'index out of range [0] with length 0'),
        (b'close({"items":[close({})],"items[0].x":99})',
         b'{"items":[{}],"items[0].x":99}', 'items[0].x',
         b'record items[0] has no field named x'),
    )
    for source, data, path, diagnostic in no_fallback:
        Path("indexed.cue").write_bytes(source)
        for args, stdin in [(["indexed.cue"], b'0'), (["-"], source), *json_inputs(data)]:
            expect_failure(run(rust, ["export", *args, "-e", path], data=stdin),
                           diagnostic, "no literal selector-key fallback")

    # Successful traversal cannot conceal invalid unselected input.
    for sibling in (b'int', b'1 & 2'):
        source = b'items:[42]\nother:' + sibling
        Path("indexed.cue").write_bytes(source)
        for args, stdin in ((["indexed.cue"], b'0'), (["-"], source)):
            expect_failure(run(rust, ["export", *args, "-e", "items[0]"], data=stdin),
                           b'cuestacean:', "whole CUE input before index")
            export(cue, [*args, "-e", "items[0]"], canonical(b'42'),
                   "Go lazy sibling exclusion", data=stdin)
    for data, diagnostic in (
        (b'{"items":[42],"other":}', b'invalid JSON:'),
        (b'{"items":[42],"other":1.0}', b'unsupported JSON:'),
        (b'{"items":[42],"other":{"x":0,"x":0}}', b'unsupported JSON:'),
    ):
        for args, stdin in json_inputs(data):
            expect_failure(run(rust, ["export", *args, "-e", "items[0]"], data=stdin),
                           diagnostic, "whole JSON input before index")


def vet_profile(rust, cue):
    """Real files, independent answers, and separate native/oracle categories."""
    schema, data = Path("vet-schema.cue"), Path("vet-data.json")
    argv = ["vet", "-c", str(schema), str(data)]
    sentinel = b'{"stdin_only":true}'

    def files(source, document):
        schema.write_bytes(source.encode() if isinstance(source, str) else source)
        data.write_bytes(document.encode() if isinstance(document, str) else document)

    def native(category, label, args=None, *, reason=None):
        result = run(rust, argv if args is None else args, data=sentinel)
        if category is None:
            require(result.returncode == 0 and not result.stdout and not result.stderr,
                    f"vet Rust {label}: {result!r}")
        else:
            expect_failure(result, category, f"vet Rust {label}")
            diagnostic = result.stderr.removeprefix(b"cuestacean: ")
            require(diagnostic.startswith(category), f"vet wrong category {label}: {result!r}")
        if reason is not None:
            require(reason in result.stderr, f"vet wrong reason {label}: {result!r}")
        return result

    def oracle(outcome, label):
        result = run(cue, argv, data=sentinel)
        if outcome == "ok":
            require(result.returncode == 0 and not result.stdout and not result.stderr,
                    f"vet Go {label}: {result!r}")
        else:
            require(result.returncode == 1 and not result.stdout and result.stderr,
                    f"vet Go {label}: {result!r}")
            # Oracle categories are asserted separately, never used as Rust answers.
            tokens = {"incomplete": (b"incomplete value",),
                      "conflict": (b"conflicting values", b"out of bound",
                                   b"incompatible number bounds", b"incompatible integer bounds"),
                      "closure": (b"field not allowed",),
                      "length": (b"incompatible list lengths",)}
            require(any(token in result.stderr for token in tokens[outcome]),
                    f"vet Go wrong category {label}: {result!r}")

    categories = {"ok": None, "conflict": b"vet conflict:",
                  "incomplete": b"vet incomplete:"}

    def parity(source, document, outcome):
        files(source, document)
        label = f"{source!r} {document!r} => {outcome}"
        native(categories[outcome], label)
        oracle(outcome, label)

    # Both literal term orders are independent source inputs, not exported DATA.
    rows = [
        ("int", ">=0", [("0", "ok"), ("-1", "conflict"), ('"0"', "conflict"),
                         ("null", "conflict"), ("true", "conflict"), (None, "incomplete")]),
        (">=0", "<=10", [("-1", "conflict"), ("0", "ok"), ("10", "ok"),
                           ("11", "conflict"), (None, "incomplete")]),
        (">0", "<10", [("0", "conflict"), ("1", "ok"), ("9", "ok"),
                         ("10", "conflict"), (None, "incomplete")]),
        (">=42", "<=42", [(None, "incomplete"), ("42", "ok"), ("41", "conflict")]),
        (">=1", "<=0", [(None, "conflict"), ("0", "conflict")]),
        ("int", "42", [(None, "ok"), ("42", "ok"), ("41", "conflict")]),
        ("bool", "true", [(None, "ok"), ("true", "ok"), ("false", "conflict"),
                           ("1", "conflict")]),
        ("string", '"A"', [(None, "ok"), ('"A"', "ok"), ('"B"', "conflict")]),
        ("int", "string", [(None, "conflict"), ("0", "conflict")]),
        ("1", "2", [(None, "conflict"), ("1", "conflict")]),
    ]
    for left, right, candidates in rows:
        for a, b in ((left, right), (right, left)):
            for value, outcome in candidates:
                term = f"{a} & {b}"
                if ("<" in term or ">" in term) and "int" not in (a, b):
                    term = f"int & ({term})"
                parity(f"age: {term}\n", "{}" if value is None else '{"age":' + value + '}',
                       outcome)
    for term in ("null", "true", "false", "0", '""', "42", "null & null"):
        parity(f"age: {term}", "{}", "ok")
    for term in ("int", "bool", "string", "int & >=0", "int & <=0", "int & >0", "int & <0"):
        parity(f"age: {term}", "{}", "incomplete")
    for term in ("int & ((>=0 & <=10) & 5)", "(5 & (<=10 & >=0)) & int"):
        parity(f"age: {term}", "{}", "ok")
        parity(f"age: {term}", '{"age":6}', "conflict")
    for term in ('int & >=42 & <=42', '<=42 & >=42 & int'):
        parity(f'age:{term}', '{}', 'incomplete')
        parity(f'age:{term}', '{"age":42}', 'ok')
    for bound, value in (('>=-9223372036854775808', '-9223372036854775808'),
                         ('<=9223372036854775807', '9223372036854775807')):
        for term in (f'int & {bound}', f'{bound} & int'):
            parity(f'age:{term}', '{"age":' + value + '}', 'ok')
            parity(f'age:{term}', '{}', 'incomplete')
    for value in ("-9223372036854775808", "9223372036854775807", "9007199254740993"):
        parity(f"age: int & {value}", '{"age":' + value + '}', "ok")
        parity(f"age: {value} & int", "{}", "ok")
    for term in ("int & >=-3 & <=-1", "<=-1 & >=-3 & int"):
        parity(f"age: {term}", '{"age":-2}', "ok")
        parity(f"age: {term}", '{"age":0}', "conflict")
    parity('age: int & < -1', '{"age":-2}', "ok")
    files('age: int & <-1', '{"age":-2}')
    native(None, "adjacent negative bound: retained Rust spelling")
    expect_failure(run(cue, argv, data=sentinel), b"found '<-'", "Go adjacent negative bound gap")
    parity('name: string & "a🦀"', '{"name":"a\\uD83E\\uDD80"}', "ok")
    for source in ('age:int & >=0\nname:string\nprovided:42\n',
                   'provided:42\nname:string\nage:>=0 & int\n'):
        for document in ('{"age":0,"name":"A","extra":[null,{"x":true}]}',
                         '{"extra":{"nested":[null]},"name":"A","age":0}'):
            parity(source, document, "ok")
        parity(source, '{"age":0}', "incomplete")
        parity(source, '{"name":"A","age":-1}', "conflict")
    parity('true:bool\nfalse:null\nnull:string', '{"true":true,"null":"x"}', "ok")

    # Shape checks on ALL known members precede any semantic evaluation.
    for source in ('age:int\nlater:string', 'age:1 & 2\nlater:string'):
        for document in ('{"later":[]}', '{"age":0,"later":{}}'):
            files(source, document)
            native(b"unsupported vet data:", "known container before semantics")
    for document in ("[]", "42", "null", '"object"', "true"):
        files("age:int", document)
        native(b"unsupported vet data:", "object root required")

    # CUE may accept these; they are explicit compatibility gaps, NOT parity.
    gaps = [
        ('age:_', '{"age":0}'), ('age:1+1', '{"age":2}'),
        ('age:>=(0)', '{"age":0}'), ('age:and([int,>=0])', '{"age":0}'),
        ('age:close({"x":1}) & close({"x":1})', '{"age":{"x":1}}'),
        ('age:*1 | 2', '{}'), ('age:int & >9223372036854775807', '{"age":9223372036854775808}'),
    ]
    for source, document in gaps:
        files(source, document)
        native(b"unsupported vet schema:" if "*" not in source else b"invalid vet schema:",
               "excluded source, Go accepts")
        oracle("ok", "excluded source, Go-only success")
    files('age:>9223372036854775807 & int', '{"age":9223372036854775808}')
    native(b"unsupported vet schema:", "bounded exclusive maximum, reversed terms")
    oracle("ok", "unbounded Go maximum, reversed terms")
    for term in ('int & < -9223372036854775808', '< -9223372036854775808 & int'):
        files(f'age:{term}', '{}')
        native(b"unsupported vet schema:", "bounded exclusive minimum")
        oracle("incomplete", "unbounded Go minimum remains incomplete")
    for term in ('>0 & <1', '<1 & >0'):
        for document, outcome in (('{}', 'incomplete'), ('{"age":0}', 'conflict'),
                                  ('{"age":1}', 'conflict')):
            files(f'age:{term}', document)
            native(b"unsupported vet schema:", "untyped interval excluded even with integer data")
            oracle(outcome, "untyped interval: Go-only domain observation")
        for typed in (f'int & ({term})', f'({term}) & int'):
            parity(f'age:{typed}', '{}', 'conflict')
            parity(f'age:{typed}', '{"age":0}', 'conflict')
    for term in ('>=0', '<=42 & >=42', '42 & >=0'):
        files(f'age:{term}', '{"age":42}')
        native(b"unsupported vet schema:", "bound needs explicit schema int, not integer candidate")
        oracle('ok', "Go-only untyped matching integer")

    # Same AST root must not turn an explicit closed schema into an open one.
    files('close({"age":int})', '{"age":0,"extra":true}')
    expect_failure(run(cue, argv, data=sentinel), b"field not allowed", "Go explicit closed root")
    for source in ('close({"age":int})', '(close({"age":int}))',
                   '// leading trivia\nclose({"age":int})', '42', '(42)', ''):
        files(source, '{"age":0,"extra":true}')
        native(b"invalid vet schema:", "no expression fallback or closed-root reinterpretation")
    for source in ('age?:int', 'age!:int', '"age":int', '_age:int', '#Age:int',
                   'package p\nage:int', 'import "strings"\nage:int',
                   'age:int\nage:int', 'age:int, name:string', 'age:ref'):
        files(source, '{}')
        native(b"invalid vet schema:", "excluded declarations")

    # Unsupported later nodes cannot hide behind early conflict/incompleteness.
    excluded = ('1+1', '>= (0)', 'close({"x":1+1})', '[1] & [1]', 'and([int])', '_', '!(true)')
    for first in ('int', '1 & 2'):
        for term in excluded:
            files(f"age:{first}\nlater:{term}", '{}')
            native(b"unsupported vet schema:", "whole schema before semantics")
        files(f"age:{first}\nlater:", '{}')
        native(b"invalid vet schema:", "later malformed schema")
    # Schema admission precedes JSON decoding, even with invalid JSON on disk.
    files('age:1+1', b'{')
    native(b"unsupported vet schema:", "schema phase before JSON phase")
    files('age:int', b'age:0')
    native(b"invalid JSON:", "explicit JSON cannot retry as CUE")
    files(b'{"age":0}', '{"age":0}')
    native(b"invalid vet schema:", "schema cannot retry as JSON")

    malformed = (b'{"other":}', b'{"other":"\\uD800"}', b'{"other":"\xff"}', b'{}{}')
    excluded_json = (b'{"other":1.0}', b'{"other":1e0}', b'{"other":-0}',
                     b'{"other":9223372036854775808}', b'{"other":{"x":0,"\\u0078":0}}')
    for first in ('int', '1 & 2'):
        for category, values in ((b"invalid JSON:", malformed),
                                 (b"unsupported JSON:", excluded_json)):
            for document in values:
                files(f"age:{first}", document)
                native(category, "whole JSON before semantics")
        for member in (b'"other":1.0', b'"other":{"x":0,"x":0}', b'"other":}'):
            for document in (b'{"age":0,' + member + b'}', b'{' + member + b',"age":0}'):
                files(f"age:{first}", document)
                native(b"invalid JSON:" if member.endswith(b':}') else b"unsupported JSON:",
                       "bad extra before/after known key")

    # Boundary fixtures calculate byte/container/node counts independently.
    limit = 1024 * 1024
    for size in (limit - 1, limit):
        source = b'age:int' + b' ' * (size - len(b'age:int'))
        files(source, '{"age":0}')
        native(None, f"schema bytes {size}")
        document = b'{"age":0}' + b' ' * (size - len(b'{"age":0}'))
        files('age:int', document)
        native(None, f"JSON bytes {size}")
    source = b'age:0\n//' + 'é'.encode() * ((limit - 8) // 2)
    require(len(source) == limit, "incorrect multibyte schema boundary")
    files(source, '{}')
    native(None, "multibyte schema at byte limit")
    files(source + b' ', '{}')
    native(b"vet setup:", "multibyte schema above byte limit")
    document = b'{"age":"' + 'é'.encode() * ((limit - 10) // 2) + b'"}'
    require(len(document) == limit, "incorrect multibyte JSON boundary")
    files('age:string', document)
    native(None, "multibyte JSON at byte limit")
    files('age:string', document + b' ')
    native(b"unsupported JSON:", "multibyte JSON above byte limit")
    files(b'age:int' + b' ' * (limit - 7) + b'\xff', '{"age":0}')
    native(b"vet setup:", "schema byte limit before UTF-8")
    files('age:int', b'{"age":0}' + b' ' * (limit - 8) + b'\xff')
    native(b"unsupported JSON:", "JSON byte limit before UTF-8")
    for depth in (63, 64, 65):
        # Outer object plus depth-1 nested arrays, in an ignored extra.
        document = b'{"age":0,"extra":' + b'[' * (depth - 1) + b'0' + b']' * (depth - 1) + b'}'
        files('age:int', document)
        native(None if depth <= 64 else b"unsupported JSON:", f"JSON containers {depth}")
    for first in ('int', '1 & 2'):
        files(f'age:{first}', b'{"extra":' + b'[' * 64 + b'0' + b']' * 64 + b'}')
        native(b"unsupported JSON:", "extra depth limit before semantics")
    for count in (62, 63, 64):
        # Record root + one literal per field; missing literals supply values.
        files('\n'.join(f'f{i}:0' for i in range(count)), '{}')
        native(None if count <= 63 else b"invalid vet schema:", f"schema AST nodes {count + 1}",
               reason=None if count <= 63 else b"expression exceeds 64 nodes")
    for length in (64, 65):
        files('a' * length + ':0', '{}')
        native(None if length == 64 else b"unsupported vet schema:", f"label bytes {length}")
    for groups in (31, 32):
        # Field RHS starts at parser depth 1; each group adds one level.
        files('age:' + '(' * groups + '0' + ')' * groups, '{}')
        native(None if groups == 31 else b"invalid vet schema:", f"schema parse depth {groups + 1}",
               reason=None if groups == 31 else b"expression exceeds 32 parsing levels")
    # Large UTF-8 scalar and wide/deep extras bypass evaluator collection caps.
    files('age:string', '{"age":"' + 'é' * 10000 + '","extra":[' + ','.join('0' for _ in range(65)) + ']}')
    native(None, "UTF-8 scalar and extra width beyond evaluator cap")

    files('age:int', '{"age":0}')
    for args in (["vet"], ["vet", str(schema), str(data)],
                 ["vet", "-c=false", str(schema), str(data)],
                 ["vet", "-c=true", str(schema), str(data)],
                 ["vet", str(schema), "-c", str(data)],
                 ["vet", "-c", "-c", str(schema), str(data)],
                 ["vet", "-c", str(data), str(schema)],
                 ["vet", "-c", "missing.cue", "-"],
                 ["vet", "-c", "missing.CUE", "missing.JSON"],
                 ["vet", "-c", "schema.cue", "data.yaml"],
                 ["vet", "-c", "-", str(data)],
                 ["vet", "-c", str(schema), "json:", "-"],
                 ["vet", "-c", ".", str(data)]):
        native(b"usage: cuestacean vet", "exact argv only", args)
    for option in ('-d', '-e', '--out=json', '--', '-c'):
        native(b"usage: cuestacean vet", "no additional flags", [*argv, option])
    for name in ('vet-missing.cue', 'vet-missing.json'):
        require(not Path(name).exists(), "missing-file fixture unexpectedly exists")
    native(b"vet setup:", "missing schema cannot retry stdin",
           ["vet", "-c", "vet-missing.cue", str(data)])
    native(b"vet setup:", "missing data cannot retry stdin",
           ["vet", "-c", str(schema), "vet-missing.json"])
    # #317: independently authored answers, not exported schema DATA or runtime
    # record equality. Native and pinned public-oracle categories stay separate.
    def nested(source, document, outcome, path=None, reason=None):
        files(source, document)
        label = f"nested {source!r} {document!r} => {outcome}"
        result = native(None if outcome == "ok" else b"vet " + (
            b"conflict" if outcome == "closure" else outcome.encode()) + b":",
            label, reason=reason)
        if path is not None:
            require(b"field " + path.encode() + b":" in result.stderr,
                    f"nested wrong native path {label}: {result!r}")
        oracle(outcome, label)

    # The formerly excluded exact close row is now successful parity.
    nested('age:close({"x":1})', '{"age":{"x":1}}', 'ok')
    person = 'person:close({"age":int,"units":"years"})'
    for document in ('{}', '{"person":{}}'):
        nested(person, document, 'incomplete', 'person.age')
    nested(person, '{"person":{"age":42}}', 'ok')
    nested(person, '{"person":{"age":false}}', 'conflict', 'person.age')
    for document in ('{"person":{"age":42,"extra":0}}',
                     '{"person":{"extra":0}}'):
        nested(person, document, 'closure', 'person.extra', b'field not allowed')
    for term in ('close({"units":"years"})', 'close({})'):
        source = 'person:' + term
        for document in ('{}', '{"person":{}}'):
            nested(source, document, 'ok')
        nested(source, '{"person":{"age":42}}', 'closure', 'person.age',
               b'field not allowed')
    for source in (person, 'person:close({"units":"years"})', 'person:close({})'):
        for value in ('null', '[]', 'true', '0', '"x"'):
            nested(source, '{"person":' + value + '}', 'conflict', 'person',
                   b'expected a record')
    deep = 'person:close({"child":close({"age":int})})'
    for document in ('{}', '{"person":{}}', '{"person":{"child":{}}}'):
        nested(deep, document, 'incomplete', 'person.child.age')
    nested(deep, '{"person":{"child":{"age":42}}}', 'ok')
    nested(deep, '{"person":{"child":null}}', 'conflict', 'person.child',
           b'expected a record')
    nested(deep, '{"person":{"child":{"extra":0}}}', 'closure',
           'person.child.extra', b'field not allowed')
    for term in ('close({})', 'close({"value":null})', 'close({"value":42})'):
        nested('person:close({"child":' + term + '})', '{}', 'ok')
    nullable = 'person:close({"value":null})'
    for document in ('{}', '{"person":{}}', '{"person":{"value":null}}'):
        nested(nullable, document, 'ok')
    nested(nullable, '{"person":null}', 'conflict', 'person', b'expected a record')
    for term in ('int & >=42 & <=42', '<=42 & >=42 & int'):
        source = 'person:close({"age":' + term + '})'
        nested(source, '{}', 'incomplete', 'person.age')
        nested(source, '{"person":{"age":42}}', 'ok')
        nested(source, '{"person":{"age":41}}', 'conflict', 'person.age')
    for term in ('int & >0 & <1', '<1 & >0 & int', '1 & 2'):
        nested('person:close({"age":' + term + '})', '{}', 'conflict', 'person.age')
    for term, value in (('bool', 'true'), ('string', '"a\\uD83E\\uDD80"'),
                        ('null', 'null'), ('int', '-9223372036854775808'),
                        ('int', '9223372036854775807'), ('int', '9007199254740993')):
        nested('person:close({"value":' + term + '})',
               '{"person":{"value":' + value + '}}', 'ok')
    mixed = 'age:int\nperson:close({"name":string,"supplied":42})'
    nested(mixed, '{"person":{"name":"A"},"age":0,"extra":[null,{"x":true}]}', 'ok')
    nested(mixed, '{"person":{"name":"A","extra":0},"age":0}', 'closure',
           'person.extra', b'field not allowed')
    nested('person:close({"\\u0061ge":int,"true":bool,"false":null,"null":42})',
           '{"person":{"age":42,"true":false}}', 'ok')

    # Exact native ordering is NOT a claim of general Go multi-error ordering.
    # Shape checks cover every known scalar before any record structure check.
    for source, document in (
        ('first:int\nperson:close({"age":int})', '{"person":{"age":[]}}'),
        ('person:close({"age":int})\nlater:close({})',
         '{"person":{"age":{}},"later":{"extra":0}}'),
        ('person:close({})\nlater:close({"age":int})',
         '{"person":null,"later":{"age":[]}}'),
    ):
        files(source, document)
        native(b'unsupported vet data:', 'recursive scalar shape before structure/semantics',
               reason=b'known fields require scalar values')
    for first in ('int', '1 & 2'):
        for value, path, reason in (('null', 'person', b'expected a record'),
                                    ('{"extra":0}', 'person.extra', b'field not allowed')):
            nested('first:' + first + '\nperson:close({})',
                   '{"person":' + value + '}',
                   'conflict' if value == 'null' else 'closure', path, reason)
    for source in ('person:close({"age":int,"child":close({})})',
                   'person:close({"child":close({}),"age":int})'):
        nested(source, '{"person":{"extra":0}}', 'closure', 'person.extra',
               b'field not allowed')
    # Go prioritizes the intrinsic contradiction; native scalar source order
    # deliberately reports earlier incompleteness. This is divergence evidence.
    files('person:close({"age":int,"n":1 & 2})', '{}')
    result = native(b'vet incomplete:', 'source-order scalar multi-error divergence')
    require(b'field person.age:' in result.stderr, 'wrong first incomplete path')
    oracle('conflict', 'Go intrinsic contradiction before missing scalar')
    nested('person:close({"n":1 & 2,"age":int})', '{}', 'conflict', 'person.n')
    for document in ('{"person":{"b":false,"a":false}}',
                     '{"person":{"a":false,"b":false}}'):
        for first, second in (('a', 'b'), ('b', 'a')):
            nested(f'person:close({{"{first}":int,"{second}":int}})',
                   document, 'conflict', 'person.' + first)
    for document in ('{"person":{"z":0,"a":0}}', '{"person":{"a":0,"z":0}}'):
        nested('person:close({})', document, 'closure', 'person.a', b'field not allowed')
    nested('person:close({"first":close({}),"second":close({})})',
           '{"person":{"second":null,"first":{"extra":0}}}', 'closure',
           'person.first.extra', b'field not allowed')
    for key, rendered in (('a.b', '["a.b"]'), ('', '[""]'),
                          ('a\\nb', '["a\\nb"]'), ('é', '["é"]')):
        files('person:close({})', '{"person":{"' + key + '":0}}')
        result = native(b'vet conflict:', 'safe unknown-key path', reason=b'field not allowed')
        require(('field person' + rendered + ':').encode() in result.stderr,
                f'unsafe/ambiguous path: {result!r}')
        require(result.stderr.count(b'\n') == 1, 'raw newline in unknown-key path')
        oracle('closure', 'unknown arbitrary JSON key')

    # Admission is recursive and complete, with no int leakage across leaves.
    terms = ('_', '1+1', '>=(0)', '[1] & [1]', 'and([int])', '!(true)',
             '>=0', 'int & >9223372036854775807',
             'close({}) & close({})', 'int & close({})')
    for first in ('int', '1 & 2'):
        for term in terms:
            files('first:' + first + '\nperson:close({"typed":int,"child":close({"bad":'
                  + term + '})})', '{}')
            native(b'unsupported vet schema:', 'whole recursive schema before semantics')
    for label in ('', '_hidden', '#Definition', 'a.b', 'é', 'a' * 65):
        files('person:close({"' + label + '":0})', '{}')
        native(b'unsupported vet schema:', 'decoded label profile')
    for source in ('person:close({"age":0,"\\u0061ge":0})',
                   'person:close({"child":close({"x":0,"x":0})})'):
        files('first:1 & 2\n' + source, '{}')
        native(b'unsupported vet schema:', 'duplicate decoded schema labels before semantics')
    nested('person:close({"child":close({"age":0}),"age":0})', '{}', 'ok')
    for source in ('person:{age:0}', 'person:close({"age"?:int})',
                   'person:close({"age"!:int})', 'person:close({"age":*1 | 2})',
                   'person:close({"age":ref})', 'person:close({"age":0 "b":0})',
                   'person:close({"age":0}', 'person:close({"age":0,})\nlater:'):
        files(source, '{}')
        native(b'invalid vet schema:', 'recursive syntax exclusions')
    files('person:close({"bad":1+1})', '{')
    native(b'unsupported vet schema:', 'recursive schema phase before JSON')
    for category, values in ((b'invalid JSON:', malformed),
                             (b'unsupported JSON:', excluded_json)):
        for document in values:
            # Even a closure violation cannot conceal invalid member contents.
            files('person:close({})', b'{"person":' + document + b'}')
            native(category, 'whole JSON before nested closure')
    for first in ('int', '1 & 2'):
        files('first:' + first + '\nperson:close({})',
              b'{"person":{"extra":' + b'[' * 63 + b'0' + b']' * 63 + b'}}')
        native(b'unsupported JSON:', 'nested extra depth before scalar/closure')

    # Aggregate expression counts include root and every (even empty) record.
    for count in (61, 62, 63):
        source = 'person:close({' + ','.join(f'"f{i}":0' for i in range(count)) + '})'
        files(source, '{}')  # total nodes = 2 + count
        native(None if count <= 62 else b'invalid vet schema:',
               f'nested aggregate AST nodes {count + 2}',
               reason=None if count <= 62 else b'expression exceeds 64 nodes')
    for count in (60, 61, 62):
        # Root + two records + count scalar children, not a per-record allowance.
        left = ','.join(f'"f{i}":0' for i in range(count // 2))
        right = ','.join(f'"f{i}":0' for i in range(count - count // 2))
        files('left:close({' + left + '})\nright:close({' + right + '})', '{}')
        native(None if count <= 61 else b'invalid vet schema:',
               f'sibling aggregate AST nodes {count + 3}',
               reason=None if count <= 61 else b'expression exceeds 64 nodes')
    for count in (62, 63, 64):
        files('\n'.join(f'f{i}:close({{}})' for i in range(count)), '{}')
        native(None if count <= 63 else b'invalid vet schema:',
               f'empty record node charges {count + 1}',
               reason=None if count <= 63 else b'expression exceeds 64 nodes')
    for groups in (30, 31):
        # person RHS record depth1, child expression depth2, then groups.
        files('person:close({"age":' + '(' * groups + '0' + ')' * groups + '})', '{}')
        native(None if groups == 30 else b'invalid vet schema:',
               f'nested parse levels {groups + 2}',
               reason=None if groups == 30 else b'expression exceeds 32 parsing levels')
    for depth in (31, 32, 33):
        # Root RHS starts at level1; each nested record adds exactly one.
        source = 'person:' + 'close({"child":' * (depth - 1) + 'close({})' + '})' * (depth - 1)
        files(source, '{}')
        native(None if depth <= 32 else b'invalid vet schema:',
               f'nested record parse levels {depth}',
               reason=None if depth <= 32 else b'expression exceeds 32 parsing levels')
    for length in (64, 65):
        files('person:close({"' + 'a' * length + '":0})', '{}')
        native(None if length == 64 else b'unsupported vet schema:',
               f'nested decoded label bytes {length}')
    # Wide decoded data is not the evaluator's closed-value width profile.
    extras = ','.join(f'"k{i}":0' for i in range(65))
    nested('person:close({})', '{"person":{},"extra":{' + extras + '}}', 'ok')
    nested('person:close({})', '{"person":{' + extras + '}}', 'closure',
           'person.k0', b'field not allowed')
    # #319: fixed positional schemas, not runtime abstract-list construction.
    # Category prefixes remain separate from path/reason assertions. List
    # diagnostics omit the old scalar/record "field " marker by contract.
    def fixed(source, document, outcome, path=None, reason=None, *, go=None):
        files(source, document)
        label = f"fixed {source!r} {document!r} => {outcome}"
        category = None if outcome == 'ok' else b'vet ' + (
            b'incomplete' if outcome == 'incomplete' else b'conflict') + b':'
        result = native(category, label, reason=reason)
        if path is not None:
            require((path + ':').encode() in result.stderr,
                    f"fixed wrong native path {label}: {result!r}")
        oracle(outcome if go is None else go, label)

    def length(term, document, expected, actual, path='x', *, go='length'):
        fixed('x:' + term, document, 'length', path,
              f'{path}: incompatible list lengths (expected {expected}, got {actual})'.encode(),
              go=go)

    # All 23 scratch-oracle observations, with multi-defect rows below kept
    # explicitly separate from isolated category/path parity.
    for document, outcome, path in (
        ('{}', 'incomplete', 'x[0]'),
        ('{"x":[1,"kg"]}', 'ok', None),
        ('{"x":[false,"kg"]}', 'conflict', 'x[0]'),
    ):
        fixed('x:[int,"kg"]', document, outcome, path)
    for document, actual in (('{"x":[]}', 0), ('{"x":[1]}', 1),
                              ('{"x":[1,"kg",0]}', 3)):
        length('[int,"kg"]', document, 2, actual)
    for document in ('{"x":null}', '{"x":{}}'):
        fixed('x:[int,"kg"]', document, 'conflict', 'x', b'x: expected a list')
    fixed('x:[42,"kg"]', '{}', 'ok')
    length('[42,"kg"]', '{"x":[42]}', 2, 1)
    for document in ('{}', '{"x":[]}'):
        fixed('x:[]', document, 'ok')
    length('[]', '{"x":[1]}', 0, 1)
    term = '[close({"n":int}),[42]]'
    for document, outcome, path, reason in (
        ('{}', 'incomplete', 'x[0].n', None),
        ('{"x":[{"n":1},[42]]}', 'ok', None, None),
        ('{"x":[{"n":1,"extra":0},[42]]}', 'closure', 'x[0].extra', b'field not allowed'),
        ('{"x":[{},[42]]}', 'incomplete', 'x[0].n', None),
    ):
        fixed('x:' + term, document, outcome, path, reason)
    length(term, '{"x":[{"n":1},[]]}', 1, 0, 'x[1]')
    # Deliberate first-error divergence, NOT clean single-defect parity.
    fixed('x:[int,1 & 2]', '{}', 'incomplete', 'x[0]', go='conflict')
    length('[int,1 & 2]', '{"x":[0]}', 2, 1, go='conflict')
    fixed('x:[int,1 & 2]', '{"x":[0,1]}', 'conflict', 'x[1]')
    for document, actual in (('{"x":[false]}', 1), ('{"x":[false,0,1]}', 3)):
        length('[int,int]', document, 2, actual, go='conflict')

    # Reclassified old list exclusions now have positive packaged coverage.
    fixed('age:[1]', '{"age":[1]}', 'ok')
    fixed('r:close({"x":[1]})', '{"r":{"x":[1]}}', 'ok')
    for candidate in ('true', '0', '"s"'):
        fixed('x:[int]', '{"x":' + candidate + '}', 'conflict', 'x',
              b'x: expected a list')
    literals = '[null,true,"a🦀",-9223372036854775808,9223372036854775807,9007199254740993]'
    for document in ('{}', '{"x":' + literals.replace('🦀', '\\uD83E\\uDD80') + '}'):
        fixed('x:' + literals, document, 'ok')
    for term in ('[int & >=0 & <10]', '[<10 & >=0 & int]'):
        for document, outcome in (('{"x":[0]}', 'ok'), ('{"x":[9]}', 'ok'),
                                   ('{"x":[-1]}', 'conflict'), ('{"x":[10]}', 'conflict'),
                                   ('{}', 'incomplete')):
            fixed('x:' + term, document, outcome, None if outcome == 'ok' else 'x[0]')
    for document, outcome in (('{}', 'incomplete'), ('{"x":[42]}', 'ok'),
                               ('{"x":[41]}', 'conflict')):
        fixed('x:[int & >=42 & <=42]', document, outcome,
              None if outcome == 'ok' else 'x[0]')
    for document in ('{}', '{"x":[[42],{}]}'):
        fixed('x:[[42],close({"a":[],"n":null})]', document, 'ok')
    for document in ('{}', '{"x":{}}', '{"x":{"items":[{}]}}'):
        fixed('x:close({"items":[close({"n":int})]})', document,
              'incomplete', 'x.items[0].n')
    fixed('x:close({"items":[close({"n":int})]})',
          '{"x":{"items":[{"n":42}]}}', 'ok')
    fixed('x:[[int]]', '{"x":[null]}', 'conflict', 'x[0]', b'x[0]: expected a list')
    fixed('x:[[int]]', '{"x":[[null]]}', 'conflict', 'x[0][0]')
    length('[[int]]', '{"x":[[]]}', 1, 0, 'x[0]')
    fixed('x:[close({})]', '{"x":[null]}', 'conflict', 'x[0]', b'expected a record')
    length('[]', '{"x":[null]}', 0, 1)

    # Native phase controls: zip only overlapping positions even when lengths
    # differ. Extra positions have JSON admission but no invented scalar term.
    for term, document in (
        ('[int,int]', '{"x":[{}]}'),
        ('[int,int]', '{"x":[{},0,1]}'),
        ('[int,int]', '{"x":[1,{},0]}'),
        ('[close({"n":int}),int]', '{"x":[{"n":[]}]}'),
        ('[[int],int]', '{"x":[[{}]]}'),
    ):
        files('x:' + term, document)
        native(b'unsupported vet data:', 'overlap scalar shapes before length',
               reason=b'known fields require scalar values')
    length('[int]', '{"x":[1,{}]}', 1, 2)
    length('[]', '{"x":[{}]}', 0, 1)
    for source in ('x:[int,int]\ny:close({"n":int})',
                   'y:close({"n":int})\nx:[int,int]'):
        for candidate in ('[]', 'null', '{}'):
            files(source, '{"x":' + candidate + ',"y":{"n":[]}}')
            native(b'unsupported vet data:', 'all reachable shapes before list structure',
                   reason=b'known fields require scalar values')
    for first in ('int', '1 & 2'):
        for source in (f'first:{first}\nx:[int]', f'x:[int]\nfirst:{first}'):
            fixed(source, '{"x":[]}', 'length', 'x',
                  b'x: incompatible list lengths (expected 1, got 0)', go='conflict'
                  if first == '1 & 2' else 'length')
            fixed(source, '{"x":null}', 'conflict', 'x', b'x: expected a list')
    # Parent structure precedes child structure; sibling structure is schema DFS.
    length('[close({})]', '{"x":[{"extra":0},0]}', 1, 2)
    for source, path, reason in (
        ('x:[[int],close({})]', 'x[0]', b'incompatible list lengths'),
        ('x:[close({}),[int]]', 'x[0].extra', b'field not allowed'),
    ):
        document = ('{"x":[[],{"extra":0}]}' if '[[int]' in source
                    else '{"x":[{"extra":0},[]]}')
        files(source, document)
        result = native(b'vet conflict:', 'schema/index structural DFS', reason=reason)
        require((path + ':').encode() in result.stderr, 'wrong structural DFS path')
    fixed('x:[1 & 2,int]', '{}', 'conflict', 'x[0]')
    for document in ('{"x":[false,true]}', '{"x":[true,false]}'):
        fixed('x:[int,int]', document, 'conflict', 'x[0]')

    for key in ('a.b', '', 'line\n', '"', 'é'):
        files('x:[close({})]', json.dumps({'x': [{key: 0}]}, ensure_ascii=False))
        rendered = 'x[0][' + json.dumps(key, ensure_ascii=False) + ']'
        result = native(b'vet conflict:', 'escaped indexed unknown-key path',
                        reason=b'field not allowed')
        require(('field ' + rendered + ':').encode() in result.stderr,
                f'wrong indexed unknown-key path: {result!r}')
        require(result.stderr.count(b'\n') == 1, 'raw newline in indexed path')
        oracle('closure', 'indexed unknown arbitrary JSON key')

    # Whole syntax admission; no flags leak from one list position to another.
    for term in ('_', '1+1', '>=(0)', 'and([int])', '!(true)', '>=0',
                 'int & >9223372036854775807', '[1] & [1]',
                 'int & [1]', 'close({}) & [1]', 'close({}) & close({})'):
        for first in ('int', '1 & 2'):
            files('first:' + first + '\nx:[close({"typed":int}),[' + term + ']]', '{}')
            native(b'unsupported vet schema:', 'whole fixed schema before semantics')
    for source in ('x:[int,>=0]', 'x:[close({"a":0,"\\u0061":0})]',
                   'x:[close({"a.b":0})]'):
        files(source, '{}')
        native(b'unsupported vet schema:', 'recursive list admission exclusions')
    for source in ('x:[int,...int]', 'x:[...int]', 'x:[*1 | 2]',
                   'x:[ref]', 'x:[{a:0}]', 'x:[close({"a"?:int})]',
                   'x:[close({"a"!:int})]'):
        files(source, '{}')
        native(b'invalid vet schema:', 'fixed syntax exclusions')
    files('x:[int,1+1]', '{')
    native(b'unsupported vet schema:', 'fixed schema before invalid JSON')
    for member, category in ((b'1.0', b'unsupported JSON:'),
                              (b'{"a":0,"\\u0061":0}', b'unsupported JSON:'),
                              (b'"\\uD800"', b'invalid JSON:'),
                              (b'{"a":}', b'invalid JSON:')):
        files('x:[int]', b'{"x":[0,' + member + b']}')
        native(category, 'whole surplus JSON before length')
    files('x:[]', b'{"x":[' + b'[' * 63 + b'0' + b']' * 63 + b']}')
    native(b'unsupported JSON:', 'surplus container depth before length')
    files('x:[int]', '{"x":[0],"extra":1.0}')
    native(b'unsupported JSON:', 'whole ignored extra before fixed semantics')

    # Independent aggregate counts: root + list + scalar elements, and
    # root + every empty list. This does not raise any parser/value bounds.
    for count in (61, 62, 63):
        files('x:[' + ','.join('0' for _ in range(count)) + ']', '{}')
        native(None if count <= 62 else b'invalid vet schema:',
               f'fixed aggregate AST nodes {count + 2}',
               reason=None if count <= 62 else b'expression exceeds 64 nodes')
    for count in (62, 63, 64):
        files('\n'.join(f'f{i}:[]' for i in range(count)), '{}')
        native(None if count <= 63 else b'invalid vet schema:',
               f'empty list AST charges {count + 1}',
               reason=None if count <= 63 else b'expression exceeds 64 nodes')
    for count in (62, 63, 64):
        files('\n'.join(f'f{i}:' + ('[]' if i % 2 else 'close({})')
                        for i in range(count)), '{}')
        native(None if count <= 63 else b'invalid vet schema:',
               f'mixed empty container AST charges {count + 1}',
               reason=None if count <= 63 else b'expression exceeds 64 nodes')
    for count in (60, 61, 62):
        left = '[' + ','.join('0' for _ in range(count // 2)) + ']'
        right = 'close({' + ','.join(f'"f{i}":0' for i in range(count - count // 2)) + '})'
        files('left:' + left + '\nright:' + right, '{}')
        native(None if count <= 61 else b'invalid vet schema:',
               f'mixed sibling aggregate AST nodes {count + 3}',
               reason=None if count <= 61 else b'expression exceeds 64 nodes')
    for depth in (31, 32, 33):
        for mixed in (False, True):
            term = '[]'
            for level in range(depth - 1):
                term = 'close({"n":' + term + '})' if mixed and level % 2 else '[' + term + ']'
            files('x:' + term, '{}')
            native(None if depth <= 32 else b'invalid vet schema:',
                   f'fixed parse levels {depth} mixed={mixed}',
                   reason=None if depth <= 32 else b'expression exceeds 32 parsing levels')
    for groups in (30, 31):
        files('x:[' + '(' * groups + '0' + ')' * groups + ']', '{}')
        native(None if groups == 30 else b'invalid vet schema:',
               f'fixed grouped parse levels {groups + 2}',
               reason=None if groups == 30 else b'expression exceeds 32 parsing levels')
    fixed('x:[string]', '{"x":["' + 'é' * 10000 + '"],"extra":['
          + ','.join('0' for _ in range(65)) + ']}', 'ok')
    print("Packaged vet parity, categories, phase/limit controls and explicit gaps verified")


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
    indexed_selection(rust, cue)
    vet_profile(rust, cue)

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
