# CUE-owned instruction-to-Rust lowering

[`lowering-fragments.cue`](lowering-fragments.cue) defines executable expression
lowering for all sixteen evaluation instructions. [`lowering.cue`](lowering.cue)
selects the normal program. CUE chooses runtime calls, constructor layout,
recursive instruction-field selection, grouping and Result propagation.
`bootstrap-codegen` no longer contains a per-instruction emission match.

This is an **interpreted generator stage**, not full generator self-hosting.
The generic policy VM, typed rule decoder, Rust syntax renderer, outer rule-map
iteration/module assembly, literal escaping and safety guards remain Rust.
There is not yet a compiler that replaces this VM with generated lowering code.
Go CUE still exports the policy. Source-language/evaluator behavior is unchanged.

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
strings remain data and cannot become paths or unescaped Rust syntax.

## Limits

- A streaming structural check counts original JSON values, including overwritten
  duplicates, before typed decoding. Original-text decoding then rejects duplicate
  record fields and case keys; generic JSON maps cannot erase that evidence.
- A policy has at most 512 JSON nodes and 32 levels, counting the root. It has
  1–64 cases; keys are 1–64 bytes, selectors 1–128 bytes and paths at most 128.
- Each instruction lowering shares a budget of 4096 dispatch/expression steps,
  at most 64 instruction levels, 128 combined dispatch/render frames, and 1 MiB
  rendered bytes, including intermediate copies. Duplicate-child expansion cannot reset these budgets.
- Selected string values are limited to 64 KiB before allocating quoted output.
  Debug quoting writes directly through the byte budget, without an uncharged
  quoted-string temporary. Output-budget checks precede appending. These are bootstrap bounds, not
  comprehensive hostile-input hardening or whole-module size limits.

## API, seeds and evidence

`generate(rules)` uses the embedded, fresh CUE JSON policy.
`generate_with_policy(rules, policy)` and
`bootstrap-codegen <rules.json> [lowering.json]` allow explicit policy selection.
Failures return before the CLI prints any partial source.

Compiler seeds now depend on **upstream-built policy JSON**. The dependency graph
is CUE sources → JSON exports → native VM/compiler seeds → generated Rust → final
package. Exports do not depend on seeds or committed Rust. Sandbox stages still
delete staged JSON/Rust; there is no handwritten instruction-policy fallback.
Generated JSON stays build-only and Rust goldens stay committed.

The normal policy reproduces all existing evaluator modules byte-for-byte.
`lowering-left-twice` changes only sum's right-child selection to lower the left
instruction twice. Normal rules plus this policy produce exactly the existing
`evaluation-left-twice.rs` module, whose compiled tests independently expect 40
instead of 42. Nix regenerates, compares and compiles that policy-produced module,
not merely a string snapshot. Tests also cover all opcodes independently, literal
escaping, duplicate/unused-invalid rejection, exact structural boundaries,
shared expansion limits, determinism and CLI policy failures.
