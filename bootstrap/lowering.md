# CUE-owned IR-to-Rust lowering

[`lowering-fragments.cue`](lowering-fragments.cue) defines executable expression
lowering for all sixteen evaluation instructions. [`lowering.cue`](lowering.cue)
selects the normal program. CUE chooses runtime calls, constructor layout,
recursive instruction-field selection, grouping and Result propagation.
`bootstrap-codegen` no longer contains a per-instruction emission match. The
[Pratt compiler backend](pratt-lowering.md) now reuses this same machinery for
numbers, predicates, statements and the generated parser-driver wrapper. The
[unification backend](unify-lowering.md) now also uses it for solver-control
emission and lexical binding layout. The [AST-builder backend](ast-lowering.md)
also uses it for constructors and ownership selection from checked native facts.

The production generator now uses a **compiled CUE lowering driver**, not the
policy interpreter. `bootstrap-lower-codegen` compiles validated policy expressions
to Rust calls into a shared checked rendering kernel. The reference interpreter
uses that same kernel for seeds and explicit custom policies. This is not full
generator self-hosting: the policy compiler, typed decoder, renderer and safety
guards remain Rust. The policy's [module statement program](module.md) now owns
outer boilerplate and rule traversal as well. Go CUE still exports policies;
source-language/evaluator behavior is unchanged.

## Typed policy

`#LoweringPolicy` version 1 maps opcode strings to expressions:

| Operation | Meaning |
| --- | --- |
| `path {value}` | Emit a fixed policy path, e.g. `runtime::Value::Int` |
| `call {path, args}` | Render a fixed function/constructor and ordered arguments |
| `ok {value}` | Wrap an expression in `Ok::<_, Error>(...)` |
| `group {value}` | Parenthesize an expression |
| `try {value}` | Append Rust `?` |
| `lower {field}` | Select a child instruction and recursively run this policy |
| `string {field}` | Read a string and quote it with Rust Debug escaping |
| `boolean {field}` | Read a bool and emit its literal spelling |
| `text {value}` | Append trusted target-source text |
| `concat {parts}` | Ordered concatenation with no separator |
| `unsigned {field}` | Stream a u64 as exact decimal |
| `signed {field}` | Stream an i64 as exact decimal |
| `list {field}` | Lower each array child with the selected driver/shared budget |
| `ifEmpty {field, then, else}` | Select a branch using array emptiness |
| `join {field, separator}` | Lower array children with trusted separators (≤128 bytes) |
| `ifZero {field, then, else}` | Select a branch using a u64 zero test |
| `select {field, cases}` | Select one trusted expression by string-field value |

The instruction is the current input to every expression; recursive `lower`
changes that input to the selected child. There is no native sum/product/etc.
case tree behind these operations. The existing typed instruction decoder still
validates the entire rule document before policy execution, even if a policy
ignores some fields. Every policy template is validated, including unused cases.

Paths originate only in the trusted implementation policy, never instruction
string fields. They allow ASCII identifier segments separated by `::`, not raw
Rust snippets. This is lexical validation, not Rust name/type resolution;
the downstream compiler checks those. Policies are executable compiler inputs,
not a security boundary for compiling untrusted implementation code. Instruction
strings remain data and cannot become paths or unescaped Rust syntax. Fixed
`text`, like module `text`, is instead trusted implementation source and may
contain arbitrary Rust syntax; it is not subject to the path grammar.

## Limits

- A streaming structural check counts original JSON values, including overwritten
  duplicates, before typed decoding. Original-text decoding then rejects duplicate
  record fields and case keys; generic JSON maps cannot erase that evidence.
- A policy has at most 512 JSON nodes and 32 levels, counting the root. It has
  1–64 cases; keys are 1–64 bytes, selectors 1–128 bytes and paths at most 128.
  Nested `select` maps have the same case/key limits and eager validation.
- Each instruction lowering shares a budget of 4096 dispatch/expression steps,
  at most 64 instruction levels, 128 combined dispatch/render frames, and 1 MiB
  rendered bytes, including intermediate copies. Duplicate-child expansion cannot reset these budgets.
- Selected string values are limited to 64 KiB before allocating quoted output.
  Debug quoting writes directly through the byte budget, without an uncharged
  quoted-string temporary. Output-budget checks precede appending. These are bootstrap bounds, not
  comprehensive hostile-input hardening. Module assembly has its own cumulative
  output/step bounds; see the [module contract](module.md).

## API, seeds and evidence

`generate(rules)` uses `compiled_lowering::Compiled` under the default
`compiled-lowering` feature. `generate_with_policy(rules, policy)` and the optional
`bootstrap-codegen <rules.json> [lowering.json]` override use the reference engine.
`lowering::generate(policy)` / `bootstrap-lower-codegen <policy.json>` compile a
policy to a Rust driver. Both compilers return before printing partial source.

Generated matches own opcode dispatch and expression sequencing; recursive
`child` calls retain the selected driver, including alternate policies. The
shared native `Context`/`Call` kernel owns lexical rendering, field projection,
quoting, ordered argument append and guards. Interpreted and compiled execution
charge each dispatch/expression, delimiter and intermediate copy identically.

Six native compiler seeds build with `--no-default-features`, excluding generated
lowering, semantics and frontend. The seed rule generator interprets fresh
**upstream-built policy JSON**; it does not contain a handwritten instruction
policy. Seeds generate normal/alternate lowering and assembly drivers and all evaluator/frontend
Rust before the final default-feature build uses compiled lowering. The final
compiled generator regenerates the same evaluator bytes in drift checks.

`compiled-lowering` is independent of `semantics`/`source`; semantics-only and
compiler-only profiles do not require the compiled driver. Sandbox feature
checks remove all lowering drivers before checking semantics-only binaries.
Exports depend only
on CUE sources, never seeds or committed Rust. Both sandbox stages delete staged
JSON/Rust first. JSON remains build-only and Rust goldens stay committed.

The normal policy reproduces all existing evaluator modules byte-for-byte.
`lowering-left-twice` changes only sum's right-child selection to lower the left
instruction twice. Normal rules plus this policy produce exactly the existing
`evaluation-left-twice.rs` module, whose compiled tests independently expect 40
instead of 42. Nix regenerates, compares and compiles that policy-produced module,
not merely a string snapshot. Tests also cover all opcodes independently, literal
escaping, duplicate/unused-invalid rejection, exact structural boundaries,
shared expansion limits, determinism and CLI policy failures. A compiled
adversarial policy checks escaped case/selector strings and empty calls. Fixed
frame/byte-boundary tests pin acceptance, unwinding and delimiter-vs-field-error
precedence independently of interpreter/compiled agreement. Reverse assembly
has its own compiled evaluator golden, without changing the normal evaluator
artifacts.
