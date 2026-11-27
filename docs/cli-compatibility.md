# Cuestacean CLI compatibility

Cuestacean implements a small `cue export` invocation profile, tested against
Go CUE v0.16.1. This is not an upstream `cue` replacement, a Go library API,
or a claim of Go-free construction. The installed Rust CLI does not invoke Go
CUE at runtime.

Build, run, or install from the repository root:

```sh
nix build ./infra/tooling#cuestacean
nix run ./infra/tooling#cuestacean -- export bootstrap/source/valid/nested.cue
nix profile install ./infra/tooling#cuestacean
```

The installed name is `cuestacean`. To use `cue` in an interactive shell without
replacing an existing executable, opt in with `alias cue=cuestacean` in that shell.
The existing `bootstrap-source` command is unchanged.

## Supported invocation profile

Both commands below print `42` followed by a newline:

```sh
printf '40 + 2\n' | cue export --out json -
printf '40 + 2\n' | cuestacean export --out json -
```

| Surface | Status |
| --- | --- |
| `export` with one explicit UTF-8 `.cue` file (case-sensitive suffix) or `-` for stdin | One supported expression or [ordinary fields with expression values](ordinary-file-fields.md). |
| `export` with one explicit `.json` file or `json: -` for stdin | One [bounded JSON value](#explicit-json-input). Bare `-` still selects CUE. |
| JSON on stdout | Default, or `--out json` / `--out=json`. `-o -`, `--outfile -`, and `--outfile=-` explicitly select stdout. |
| Argument order | Supported options may precede or follow the input. Use `--` to end option parsing. Duplicate options and multiple inputs are rejected. |
| Field selection | `-e path`, `--expression path`, or `--expression=path` selects a value along a field and list-index path through fully concrete data. |
| Other `export` options | Unsupported, including `--escape`, `--force`, injection, `--list`, `--merge`, `--name`, package/path/schema flags, protobuf options, `--with-context`, and global error/ignore/simplify flags. Non-JSON formats and output files are unsupported. |
| `vet -c schema.cue data.json` | Validate one JSON object against a [scalar, closed-record, and fixed-list schema](#vet-schemas). |
| Other commands | Unsupported: `cmd`, `completion`, `def`, `eval`, `fix`, `fmt`, `get`, `help`, `import`, `login`, `mod`, `trim`, `version`. |

There is no implicit current-directory, package, module, import, glob, or
multi-file loading. `value: 42` exports `{"value":42}`, but references between
fields and duplicate-field unification are unsupported. See the
[source frontend](../bootstrap/source/README.md)
for language and resource limits. The existing expression evaluator has
known semantic boundaries, including checked 64-bit integers; accepting the
same invocation does not establish general language parity.

For example, both commands select `42`:

```sh
printf 'close({"answer": 42, "other": 99})\n' | cue export -e answer -
printf 'close({"answer": 42, "other": 99})\n' | cuestacean export -e answer -
```

Start with a field name, then use dotted field names or list indices, as in
`-e scope.items[0].answer`. Field names start with an ASCII letter and contain
only ASCII letters, digits, and underscores. `true`, `false`, and `null` cannot
start a path but can be field names after a dot. A list index is a nonnegative
decimal integer up to 268435454, without a leading zero unless it is `0`.
Bracket steps apply only to lists. Quote paths containing brackets in the shell.

Repeated selections, whitespace in paths, quoted labels, slices, computed
indices, hidden fields, definitions, and general expressions are unsupported.
Field lookups are exact: the full path is never looked up as a literal key.
Missing fields, out-of-range indices, and intermediate values of the wrong
container type are errors; a terminal `null` is a successful selection.
Unlike Go CUE, Cuestacean must evaluate the entire record first: even an unrelated
incomplete or conflicting field prevents selection.

## Explicit JSON input

Use `cuestacean export data.json` or pipe an export directly into
`cuestacean export json: -`, without converting it to CUE. Input must be one
UTF-8 JSON value, at most 1 MiB including whitespace, with at most 64 nested
arrays and objects (the outermost container counts as one). Strings, booleans,
null, arrays, and objects with unique decoded member names are supported.
Numbers must be decimal integers in the signed 64-bit range, without a fraction,
exponent, or negative-zero spelling. Repeated keys are rejected instead of
being overwritten or unified.

Malformed input reports `invalid JSON:`; inputs outside the supported profile
report `unsupported JSON:`. The byte limit is checked before UTF-8 validation.
Within that limit, invalid UTF-8 is rejected before parsing. Parsing stops at
container level 65 without inspecting its contents, so malformed syntax or
Unicode escapes past that point need not be diagnosed.

Go CUE accepts some inputs outside this profile, including floats, larger
integers, negative zero, and some repeated keys. Cuestacean does not try another
format when explicit JSON is invalid or unsupported. Bare `-` continues to
parse CUE and is not a portable way to feed JSON into Cuestacean. Use `-e path`
to select a value after the entire JSON input has been decoded and validated.

## Vet schemas

For example, `cuestacean vet -c schema.cue data.json` accepts
`{"age":42}` against `age: int & >=0`. Success is silent. The only supported
form takes `-c`, one explicit `.cue` schema, then one explicit `.json` instance.
There is no stdin, package loading, schema selection, or `-c=false`.

The schema must be ordinary top-level fields, not a standalone `close(...)`
expression. Within a field, use explicit `close({...})` for a closed record,
for example `person: close({"age": int})`. Nested labels must be quoted in
source and decode to names matching `[A-Za-z][A-Za-z0-9_]*`, at most 64 bytes
and unique within each record. A list such as `items: [int, "kg"]` constrains
exactly two positions. Lists and closed records can nest within one another.
Open list tails, collection conjunctions, and bare nested records are
unsupported. Scalar constraints may be scalar literals,
`int`, `bool`, `string`, literal integer bounds, or combinations of these with
`&`. A field with a bound must also contain an explicit `int` constraint; an
untyped CUE bound can admit nonintegers. Computed bounds and exclusive bounds beyond the signed 64-bit domain are unsupported. Write a
space after `<` before a negative literal (`< -1`): Cuestacean also accepts
`<-1`, but Go CUE does not. Supplied values must have the expected shape:
scalars for scalar fields, JSON objects for closed records, and arrays of exactly
the declared length for list fields. A short array is not filled from the schema.
Extra fields
are allowed at the root but forbidden inside closed records. The entire JSON
input, including extras, must still pass the JSON input checks.

A concrete schema value can supply an absent field. An absent field without a
concrete result is incomplete. Incompatible values, types, or bounds are
conflicts. Missing lists and records are checked recursively: concrete children
can supply values, but abstract children remain incomplete. Explicit `null` is
not absence. Cuestacean admits both inputs, checks known scalar positions for
unsupported container values, then checks collection structure before evaluating
scalar leaves in schema order. For a wrong-length array, the shape check visits
only positions present in both the schema and data; the JSON decoder still checks
every value. The first failure can differ from Go CUE when several constraints
fail. For example, Cuestacean may report a length mismatch before a conflicting
list element.

Both files are limited to 1 MiB. The schema retains the ordinary parser limits,
including 64 AST nodes and 32 parsing levels. JSON retains the limits described
above. Validation uses the existing generated evaluator; it does not run Go CUE.

## Output and failures

Successful exports terminate with a newline. The check compares representative
scalars byte-for-byte and compares lists and records as JSON values, not by
indentation or field order. Invalid arguments or input, conflicts, and incomplete
results return status 1, no stdout, and a diagnostic on stderr. Diagnostic
wording need not match Go CUE. Unsupported features never fall back to Go CUE.
