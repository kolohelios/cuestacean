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
| Field selection | `-e name`, `--expression name`, or `--expression=name` selects one field from a fully concrete record. |
| Other `export` options | Unsupported, including `--escape`, `--force`, injection, `--list`, `--merge`, `--name`, package/path/schema flags, protobuf options, `--with-context`, and global error/ignore/simplify flags. Non-JSON formats and output files are unsupported. |
| Other commands | Unsupported: `cmd`, `completion`, `def`, `eval`, `fix`, `fmt`, `get`, `help`, `import`, `login`, `mod`, `trim`, `version`, `vet`. |

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

Use a single ASCII field name starting with a letter and containing only letters,
digits, and underscores. `true`, `false`, and `null` are literals, not selectors.
Repeated selections, paths, quoted labels, hidden fields, definitions, and general
expressions are unsupported. A missing field or a non-record result is an error.
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

Go CUE accepts some inputs outside this profile, including floats, larger
integers, negative zero, and some repeated keys. Cuestacean does not try another
format when explicit JSON is invalid or unsupported. Bare `-` continues to
parse CUE and is not a portable way to feed JSON into Cuestacean. Use `-e name`
to select a field after the entire JSON input has been decoded and validated.

## Output and failures

Successful exports terminate with a newline. The check compares representative
scalars byte-for-byte and compares lists and records as JSON values, not by
indentation or field order. Invalid arguments or input, conflicts, and incomplete
results return status 1, no stdout, and a diagnostic on stderr. Diagnostic
wording need not match Go CUE. Unsupported features never fall back to Go CUE.
