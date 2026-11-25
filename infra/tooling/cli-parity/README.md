# Explicit JSON CLI evidence

`check.py` runs the packaged Rust CLI and pinned Go CUE v0.16.1 with an empty
runtime PATH. It feeds original producer stdout directly to both consumers as
`export json: -`, covering all four pairs against `portable.json`, an independent
expected value. No normalization occurs between producer and consumer.

The JSON DATA profile is one UTF-8 value, at most 1,048,576 input bytes and 64
nested containers (scalar depth 0, root array/object depth 1). Member names must
be unique after JSON escape decoding. Numbers are canonical signed-i64 decimal
integers: no negative zero, fractions or exponents. File and typed-stdin cases
cover values, unchanged selectors, limits, malformed input (`invalid JSON:`),
profile exclusions (`unsupported JSON:`) and no CUE fallback/sniffing. Failures
must return status 1 with empty stdout. File cases receive a distinct valid JSON
stdin sentinel, while typed-stdin cases receive the fixture bytes. A missing
JSON file with valid stdin must fail without retrying that stdin. Selection must
not hide malformed or excluded unselected members; invalid prefixes and extra
documents remain malformed even when followed by over-depth containers.

Go-only acceptance of fractions, larger integers, negative zero and equal
member duplicates is exclusion evidence, not successful parity. Unequal duplicate
members conflict upstream. Go also conflates canonically equivalent Unicode
member names such as `é` and `e\u0301`; the profile uses decoded string equality,
not Unicode normalization, so Rust accepts these distinct names. This upstream
rejection is tested separately, not counted as parity.

The existing CUE exports, bare-stdin Rust rejection and nested-default gaps stay
as controls. Comparison preserves exact integers and distinguishes booleans,
integers and strings; it ignores output layout and member order only.
