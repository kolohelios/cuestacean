# Bootstrap closure inventory (#163)

## Decision and scope

The first no-Go-CUE target is **exporting the existing AST atom builder** from
exactly `bootstrap/ast-schema.cue` and `bootstrap/ast-atom.cue`. This #163 inventory
isolated the closure and characterized upstream behavior. Following the reviewed
#162–#165 findings and user agreement under #161, [#169](native-spec-export.md)
adds an isolated native exporter/artifact path. The [#171 family step](native-builder-family.md)
now makes independently native artifacts the production provider for all 12 AST
builder specifications. Other exports and the six compiler policies still depend
on Go CUE; wider feature expansion stays deferred. The later
[#182 demand-policy cut](native-demand-policies.md) natively constructs the two
experimental dispatch policies from their unchanged three-file raw closure. They
are test-only graph policies, not any of the six lowering-policy exports; their
native artifact proof and separate Go CUE comparison are kept distinct.
[#184](preference-projection-policy.md) explicitly extends that experimental
family to four programs/six raw files for scoped flat projection choice, with a
fresh full construction-graph proof. Original production cuts remain unchanged.

The [solver spike](solver-spike.md) demonstrated bounded dependencies and separate
default projection, not general schema evaluation. The chosen target avoids
specification defaults, imports and comprehensions. It still needs named recursive
schema constraints, closedness, disjunctions and unification; it is not merely
JSON parsing or calling the eager expression VM.

## What actually depends on what

The authoritative export manifest is `infra/tooling/flake.nix`: `exportFixtures`,
`builderFixtures`/`builderExportFixtures`, `cueSources`, and `jsonExports`' exact
native-provider membership and per-CUE-fixture fragment selection. AST positive
exports come from `nativeBuilders`; the production bundle copies them and does
not run Go CUE for those fixtures. Distinguish three closures:

1. The Nix JSON derivation contains listed source files plus the independently
   produced native builder artifacts; the native family has its own minimal cut.
2. Each remaining `cue export` process loads **only** its selected schema, explicit fragment
   files and fixture, plus their imports. Package names do not load every other
   file in the directory when the command supplies an explicit file list.
3. Within those files, definitions reference further definitions; a reachable
   schema alternative is not proof its builtin executes for every concrete input.
   Parsing/loading a file is also not the same as evaluating just one definition.

Representative actual edges (paths below are relative to `bootstrap/`):

| Export root | Explicit input files beyond root | Important transitive edges |
| --- | --- | --- |
| `literal.cue` | `schema.cue` | `#Program.entry -> #Expression`; root additionally refines `#IntegerLiteral`. The expression union reaches recursive collection definitions and `list` validators, not just the integer definition. |
| `ast-atom.cue` | `ast-schema.cue` | `#BuildProgram -> #BuildParameter, #BuildExpression`; expression union reaches argument/string/integer/object; object field values recurse to expression. No imports. |
| `whitespace.cue` | `scanner-schema.cue`, `scanner-fragments.cue` | `#ScanProgram.body -> #ScanStatement`; `#WhitespaceBody` composes statement/predicate data. |
| `integer-literal.cue` | `scanner-schema.cue`, `decimal-fragments.cue` | `#IntegerSpellingScanner` composes sign/radix digit runs; `#ScanProgram` recursively checks represented control. |
| `identifier.cue` | `scanner-schema.cue` | File-local `let` byte tables call `list.Range`/`list.Concat`; scanner schema constrains the result. |
| `primary.cue`, `pratt.cue` | `schema.cue`, corresponding `*-fragments.cue` | `#PrimaryControl`/`#PrattControl -> #PrattProgram`; hidden flags/default minima affect actual conditional list generation. Record/conjunction primary and item controls have separately selected fragment files. |
| `logical-and.cue` | `schema.cue`, `equality-fragments.cue`, `logic-fragments.cue` | `#LogicProgram -> #BinaryControlProgram`; parameterized `#ConcreteSide.condition -> #UnifySide`, in the other fragment. This is a genuine multi-file edge. |
| `evaluation.cue` | `schema.cue` | `#EvaluationRules -> #EvaluateChild`; export constructs instructions, not the arithmetic result of an input program. |
| `lowering.cue` and five `*-lowering.cue` selectors | `schema.cue`, matching `*-lowering-fragments.cue` | Instruction/Pratt/unification/AST/scanner/meta definitions refine `#LoweringPolicy`, whose cases recurse through `#LoweringExpression`, path constraints and module statements. |

The `case` table includes alternate/previous policies and source controls, not only
these examples. A missing-reference diagnostic is a setup failure, never a valid
negative-schema result. Invalid fixtures explicitly load expression, AST and
scanner schemas; the three AST negative fixtures are additionally checked with
the small AST schema **alone**.

### Actual specification-language requirements

| Construct | Traceable evidence | Work not supplied by today's source evaluator |
| --- | --- | --- |
| Structs, definitions, closedness, embedding | `ast-schema.cue:#BuildProgram/#BuildObject`; `ast-atom.cue` embeds `#BuildProgram & {...}`. `tests/valid/grammar-no-brackets.cue` uses actual `close(...)`. | Declaration environment; recursive definition instantiation; field admission/closedness and conjunction, not concrete record equality. |
| Scope, field references, `let`, template reuse | `equality-fragments.cue:#ConcreteSide` specializes `_side` independently; `lowering-fragments.cue:#InstructionLowering` binds `Node`, `L`, `R`, `Unary`, `Binary`. | Lexical bindings and fresh instances; no accidental sharing of specialized fields. DSL `bind/get/letBounds` instructions are a different language. |
| Ordinary/optional/required fields | `schema.cue:#OptionalSliceExpression` has `lower?`/`upper?`; `#LoweringExpression` conditionally constrains `upperFirst?`. `ast-schema.cue` supplies omitted singleton `formatVersion: 1`. | Presence/completeness semantics and bottom checks. No explicit `!:` declarations were found in the inspected export inputs; ordinary fields still impose constraints and can contribute values. |
| Disjunctions and defaults | Recursive tagged unions in both schemas; `schema.cue:#PrattProgram` defaults `target` to `"parser"`; `primary-fragments.cue:#PrimaryControl` has multiple preferred flags/minima; `scan-lowering-fragments.cue` has overridable preferred source templates. | General default calculus, including non-integer/list/string/struct choices and interacting preferences. The single-integer preference spike is insufficient. Singleton insertion is not default selection. |
| Open lists and list constraints | `ast-schema.cue:#BuildObject.fields`, `#BuildProgram.parameters`; `scanner-schema.cue:#ScanByteIn` has a required first item plus open tail; collection shapes use `list.MaxItems`. | Typed list constraints and recursive validation, not only immutable concrete list values. |
| Conditionals/comprehensions | `pratt-fragments.cue` conditionally emits postfix clauses; `decimal-fragments.cue` flattens sign/digit programs with `for`; `schema.cue` checks optional presence via `!= _\|_`. | Export-time guards, bottom/presence, iteration and field/list generation. `{op:"while"}` alone is not evidence of a CUE loop. |
| Dynamic labels, patterns and interpolation | `lowering-fragments.cue` builds aliases/cases with `for name, method ... {"\(name)": ...}`; `schema.cue:#LoweringPolicy` constrains `[string & !=""]` fields. | Pattern/dynamic-field application, scope and string interpolation. A builder's represented dynamic JSON key is not a CUE dynamic label. |
| Imports and builtins | Export inputs import `list`; actual calls include `Range`, `Concat`, `MaxItems`, `UniqueItems`. `schema.cue:#RecordEntry` uses builtin `len`; record checks use hidden `_bytes`/`_unique` fields. | Checked builtin implementations and their constraint/error semantics, import resolution and hidden-field export rules. |
| Numbers and strings | Signed i64 extrema/bounds and `uint` constraints in `schema.cue`; actual multiline strings, concatenation and interpolation in `lowering-fragments.cue`; regex constraints in schema/grammar; Unicode/escaped strings in scanner policy data. | Exact numbers and CUE string/regex semantics. Scanner byte tables describe source recognition; they do not make the specification exporter perform radix conversion. |

Bounded search covered `bootstrap/*.cue` and `bootstrap/tests/{valid,invalid}/*.cue`,
including the manifest's specification inputs. It found only the `list` import;
this is **not** a claim about all CUE programs or separate oracle directories.
Fractional literals occur in negative schema fixtures (`bound-fraction.cue`,
`nested-non-integer.cue`); radix/exponent/SI arithmetic was not established as a
positive requirement by that search. Do not infer absence from strings containing
emitted Rust, or broaden source compatibility merely to match a syntax inventory.

### Specification evaluation versus execution of represented programs

Go CUE resolves the constructs above to produce concrete JSON. For example,
`let R = ... field: *"right" | "left"` in `lowering-fragments.cue` needs actual
CUE binding/default semantics. The resulting `{op:"lower",field:"right"}` is
then an instruction for a **Rust** lowering VM/compiler. Likewise an exported
`{op:"argument",name:"kind"}` does not refer to a CUE variable named `kind`.

Native typed decoders, validators, metadata projection and checked kernels still
participate after export. Policy execution progressively lives in the CUE-hosted
DSL; reproducing its JSON does not by itself close the compiler/bootstrap loop.
See the [status and component contracts](bootstrap-status.md).

## Current six-seed/artifact graph and Go CUE cut

```text
AST builder closures --native export twice--+
other specification closures --Go CUE-------+--> build-only JSON
                                                            |
artifact-free native Rust sources --no-default-features------+
                         |                                  |
                         +--> six fresh compiler binaries <--+
                                      |
                   JSON policies/programs interpreted and compiled
                                      |
                              fresh generated Rust
                                      |
                    final package + compiled regeneration/drift
```

`codegenSeed` deletes staged fixture JSON and `generated/*.rs`, builds exactly:

| Binary | Fresh policy selected in seed mode | Native selector |
| --- | --- | --- |
| `bootstrap-codegen` | `lowering.json` | `executor/src/codegen.rs` |
| `bootstrap-scan-codegen` | `scan-lowering.json` | `executor/src/scanner.rs` |
| `bootstrap-ast-codegen` | `ast-lowering.json` | `executor/src/ast.rs` |
| `bootstrap-pratt-codegen` | `pratt-lowering.json` | `executor/src/pratt.rs` |
| `bootstrap-unify-codegen` | `unify-lowering.json` | `executor/src/unification.rs` |
| `bootstrap-lower-codegen` | `meta-lowering.json` | `executor/src/lowering.rs` |

The seed package embeds fresh exported policies through
`CUESTACEAN_BOOTSTRAP_JSON`; no-default-features is **artifact-free**, not
**Go-CUE-independent**. Production `compiled-lowering` selects generated drivers;
`source` enables `semantics`. The final sandbox also checks compiler-only,
semantics-only and compiled-lowering isolation. Committed Rust is a drift baseline,
never a seed input. See `clearArtifacts`, `codegenSeed`, `executor`, and
`checks.bootstrap-codegen` in the flake and the [generation runbook](../bootstrap/generated/README.md).

Go CUE separately remains in source differential tests, value probes and solver
spike oracles. `bootstrap-source` normalizes collection JSON with `cue import` and
`encoding/json.Marshal`, preserving integer values. Those verification dependencies
must be accounted for separately from the specification-export path.

Export JSON currently passes through pinned `jq -S .`: recursively sorted keys,
two-space indentation, trailing LF. Pinned jq 1.8.2 preserved probes at both i64
extrema, 2^53+1 and 2^63 under this exact filter; no rounding defect was observed.
That is not permission to normalize numbers through floating-point/arithmetic
filters. Repeating a pipeline alone proves neither semantic correctness nor full
numeric fidelity. Comparisons should parse exact integers/strings and preserve
array order, with an independently specified deterministic byte format.

## First target: the AST atom builder export

The schema extraction in this change moves **seven unchanged definitions** out of
`schema.cue`, removing them from the original file rather than duplicating them.
All 11 normal/alternate builders plus `tests/valid/ast-literal` select the isolated
schema. Pre-extraction and post-extraction raw exports matched byte-for-byte for
all 12. The complete JSON export tree also matched the previous Nix output;
generated Rust drift checks remained unchanged.
This eliminates unrelated parsing/import obligations without pretending a selected
definition erased them from a still-loaded large file.

Exact source inputs, with no source transformation or pre-exported representation:

- `bootstrap/ast-schema.cue`: all seven definitions, same `package bootstrap`.
- `bootstrap/ast-atom.cue`: unchanged real builder program.

```text
#BuildProgram -> #BuildParameter, #BuildExpression
#BuildExpression -> #BuildArgument | #BuildString | #BuildInteger | #BuildObject
#BuildObject.fields[].key -> #BuildString | #BuildArgument
#BuildObject.fields[].value -> #BuildExpression  (recursive)
```

The schema cycle is a recursive **constraint template**, instantiated against a
finite object tree. It is not the same problem as the spike's cyclic arithmetic
value equations; its SCC classifier cannot simply be reused to reject this cycle.
No `let`, hidden fields, imports, defaults, optional fields, comprehensions or
interpolation occur in these two files. They need package/definition resolution,
closed structs, open typed lists, literal/type constraints, string inequality,
tagged disjunctions and embedding/unification. Unsupported **demanded** behavior
must reject explicitly, never be silently skipped or delegated to Go CUE.

Independent expected exported JSON (object key order insignificant here):

```json
{"formatVersion":1,"parameters":[{"name":"kind","type":"string"}],"result":{"op":"object","fields":[{"key":{"op":"string","value":"kind"},"value":{"op":"argument","name":"kind"}}]}}
```

The omitted `formatVersion` is supplied by a singleton constraint. Argument/key
names are data; this export does **not** build an AST node or execute the builder.
A later `bootstrap-ast-codegen` invocation consumes the result with its separate
lowering policy. That downstream compiler is outside the first attested path.

`executor/tests/bootstrap_closure.rs` records a mandatory pinned **upstream**
characterization, not native implementation evidence: independent expected JSON,
repeatable bytes, a key rename, invalid parameter names/types/operations, closedness,
incompleteness and the three isolated AST-negative fixtures. A `"missing"` argument
name deliberately exports successfully: binding validation belongs to the later
AST compiler, not this CUE schema. Future native tests must preserve that boundary,
vary declaration/input order and recursive object shape, and avoid hardcoding the
fixture's output. Numeric comparisons must remain exact when other builders join.

Run the current oracle evidence:

```sh
nix develop "path:$PWD?dir=infra/tooling" --command cue export \
  bootstrap/ast-schema.cue bootstrap/ast-atom.cue --out json
nix develop "path:$PWD?dir=infra/tooling" --command cargo test \
  --manifest-path bootstrap/executor/Cargo.toml --test bootstrap_closure
```

### Success and dependency proof for the future native milestone

Allowed trusted bootstrap dependencies: pinned Nix/Rust toolchain and locked Rust
libraries, explicitly selected handwritten parser/storage/kernel sources and these
two CUE files. A native bootstrap implementation is allowed initially, but it must
be identified as such; policy ownership/progression is a #164 decision, not a claim
that a permanent handwritten exporter achieves the project goal.

Required evidence, defined by this inventory and implemented for the isolated
path in the [native-export contract and checks](native-spec-export.md):

1. Build the exporter from an explicit source fileset excluding committed/generated
   Rust, generated JSON, existing policy-export outputs and expected-answer files.
   No `CUESTACEAN_BOOTSTRAP_JSON` or `jsonExports` dependency. Existing six compiler
   binaries cannot be relabeled as independent: their embedded policy dependency
   must first be removed/accounted for. Selecting a separate small package versus
   a checked feature boundary remains an implementation decision.
2. An actual isolated native-export derivation receives only the independently
   built exporter and the two raw CUE files. Restrict environment/PATH, deny network
   and ambient checkout access, remove staged generated artifacts. No fallback,
   cached CUE result, filename-specific answer or embedded pre-exported schema.
3. Inspect full **build derivation dependencies**, not merely runtime output
   references: no Go CUE executable/library/source or Go-CUE-exported policy may
   enter the exporter/export derivations. Use `nix derivation show --recursive`
   and recursive path inspection; retain derivation identities and build logs.
   Actually rebuild in a sandbox, not just substitute a cached result.
4. Keep the Go CUE oracle in a **separate comparison derivation**, never an input
   to exporter construction/execution. Compare both tools to handwritten JSON and
   each other with exact values, then separately test deterministic native bytes.
   Invalid/incomplete/unsupported cases must not publish successful partial JSON.
5. Demonstrate use of the native JSON as builder-compiler input in a downstream
   check; existing Go-CUE-dependent lowering/oracle machinery is allowed **there**
   only if visibly outside the attested native-export closure. Do not call that
   check an entirely Go-CUE-free generation pipeline.

### Validation of this inventory/refactor

On x86_64-linux, all three characterization tests pass in default and
no-default-features profiles; the complete Rust suite and strict all-target Clippy
pass. Pinned preflight passes 13/13 (7 inapplicable checks skipped), with actual
sandbox exports, six fresh artifact-free compiler binaries, generation/drift,
feature profiles and upstream comparisons. Two focused read-only reviews covered
closure/construct traces, extraction and attestation/provenance. Incompatible
aarch64 systems were omitted, not tested. These checks validate the inventory and
unchanged export boundary, not a native exporter or broader source capabilities.

## Subsequent closure milestones, not an implementation authorization

1. The [#171 builder-family step](native-builder-family.md) extends native export
   to every current AST builder and makes it the production provider, including
   nested objects, integer payloads and alternate versions/escaped names. It
   retains export-versus-DSL validation boundaries and the original atom cut.
2. The [#194 scanner-schema step](scanner-schema-closure.md) isolates the scanner
   schema unchanged, selects unmodified `group-open.cue` as the first exact two-file
   target and characterizes remaining native gaps. [#196](native-prefix-lists.md)
   adds native boolean values/types and required-prefix list constraints, but
   leaves the full scanner schema unsupported. Scanner construction is still Go-CUE-backed. Wider scanner/import closure (for example `identifier.cue`) and
   compiler-policy closure need explicit list/regex/default/let requirements; the
   scalar-default studies do not implement their list/string/struct preferences.
3. Close the six lowering-policy exports, including the self-regenerating meta
   policy, before claiming the six compiler seeds no longer depend on Go CUE.
4. Replace the complete specification-export cut and repeat fresh artifact-free
   generation, profiles/drift and independent upstream checks. Test-only Go CUE
   may remain an oracle without being a bootstrap dependency.

The audit should choose/check the ordering and CUE/native interfaces; the readable
walkthrough should make these distinct cuts understandable. Completing one export
is neither full self-hosting nor full CUE conformance, and no benchmark is implied.

## Initial upstream txtar selection and legal provenance

Pinned Nixpkgs `ac588ef1c914865dad517b337ea0eb9694a4021c` selects cue-lang/cue
v0.16.1, commit `6d609d768f1686f9a3a2a20197cacdbb70e5c79d`, source NAR hash
`sha256-mTj3XMWByNrKjm+/MOQGLyUKIv4JJ8i6Oaphbzls84U=`. The built executable
reports `(devel)` and CUE language v0.16.1; do not infer a different revision from
that display string.

Proposed bounded selections, inspected in that source but **not imported or run
as a native corpus**:

| Path in cue-lang/cue at the pinned commit | Select first | Purpose |
| --- | --- | --- |
| `cue/testdata/resolve/027_new-style_definitions.txtar` | `in.cue` declarations `#Foo` and quoted `"#Foo"`, excluding `bulk`, and corresponding entries in `out/evalalpha`/`out/eval` | Definition identity versus an ordinary string label. |
| `cue/testdata/resolve/025_definitions.txtar` | `in.cue` declarations `#Foo`, `foo`, `foo1`; separately the two `#Foo1` declarations, with corresponding `out/evalalpha` results/errors | Recursive closedness/unknown fields and repeated-definition conjunction. Do not adopt unrelated pattern/hidden-field sections wholesale. |
| `cue/testdata/resolve/010_optional_field_resolves_to_incomplete.txtar` | Entire `in.cue` and `out/evalalpha` | Optional references remain incomplete: an early negative boundary, not a claimed first-target capability. |

These upstream eval dumps are semantic references, not JSON-export byte goldens.
Pin extracted subcases and independent expected classifications before a native
port; do not report the complete original txtars as passing.

The pinned source has Apache-2.0 `LICENSE`; its README identifies Copyright 2018
The CUE Authors. No NOTICE file was found in that inspected source tree. Later
copying/adaptation must retain applicable notices/license, record revision/path/
section and identify modifications in visible provenance. This inventory copies
no upstream test files; full corpus adoption is separate work.
