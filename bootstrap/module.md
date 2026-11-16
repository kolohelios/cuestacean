# CUE-owned evaluator module assembly

The `module` statement program in [`lowering-fragments.cue`](lowering-fragments.cue)
owns evaluator boilerplate, rule traversal and match-arm sequencing. It is part
of the [lowering policy](lowering.md), interpreted for seeds/custom policies and
compiled into the production driver by the same sixth compiler. There is no
native header/footer or rule-emission loop in `codegen.rs` anymore.

Native code still validates typed instructions/policies, projects canonical data,
quotes data and guards cursors, output and fuel. The
[meta compiler](meta-lowering.md) now owns driver shell and module-statement source
emission too; native `module::emit` is removed. This is not full generator
self-hosting or general CUE support.

## Statements and state

| Statement | Meaning |
| --- | --- |
| `text {value}` | Append trusted implementation source text |
| `firstRule` / `lastRule` | Select a cursor endpoint, or exhausted for no rules |
| `nextRule` / `previousRule` | Step the current cursor, exhausting at the boundary |
| `kind` | Append the current rule key as a Rust Debug string literal |
| `expression` | Lower the current instruction using the selected driver |
| `whileRules {body}` | Test cursor availability and repeat body |

The cursor starts unselected. Reading or advancing an unselected/exhausted
cursor fails with `no current rule`. The native frame does not choose traversal
direction, reset the cursor inside loops or emit match arms on its own. CUE owns
those operations and their order. `expression` retains the same compiled or
reference driver for recursive instruction lowering.

`module` is optional for expression-only policies. Attempting assembly without
it returns `missing module program`, with no native boilerplate fallback.
An explicit empty list intentionally emits no source; explicit null is invalid.
All statements, including nested/unexecuted bodies, validate before compilation
or interpretation. Duplicate/unknown fields use the original policy decoder.
Compiler metadata serializes module `text` as internal `moduleText` to distinguish
it from expression text. This is projection only: wire `text` is unchanged and
wire `moduleText` rejects; no new module-language opcode is admitted.

Literal `text` is trusted implementation code, unlike rule keys/instruction
strings. It may contain Rust syntax and is not an untrusted compilation sandbox.
Rule keys always use budget-aware Rust quoting; instruction fields retain the
lowering kernel's checked literal rendering. Raw rule data cannot become module
text or paths.

## Bounds and bootstrap

A module has at most 64 statements and 16 statement levels, with each literal
at most 64 KiB. Existing global policy limits (512 JSON nodes/32 levels) also
apply. Each assembly has 4096 steps and 1 MiB total output. Every statement costs
one step; `whileRules` charges every test, including the terminating false test.
A stalled cursor cannot reset fuel. Rule keys are limited to 64 KiB before
streaming quoting through the remaining output budget.

Compiling a policy now has a separate whole-driver rendering context, including
shell and all module statements (4096 steps, 128 frames, 1 MiB including copies).
Combining individually compilable modules/cases can fail this new global emission
bound without invalidating reference assembly. See the meta contract for this
intentional compiler-boundary change and override-root requirements.

Instruction lowering retains its separate shared recursion/work/output limits.
Its bounded result is charged again when appended to the module; many individually
valid expressions cannot bypass the module cap. Failed assembly returns no partial
source through the CLI. Typed input decoding is not comprehensively hardened.

Six fresh seeds still exclude all generated Rust. The interpreted default
program produces the same normal evaluator bytes as before; final compiled
assembly reproduces those goldens. The reverse policy selects last/previous
instead of first/next, producing descending match arms with unchanged semantics.
Nix regenerates and compiles `evaluation-reverse.rs`, with independent integer,
boolean and failure tests. Cursor/trust/validation, empty/missing modules, stalled
loops, cumulative byte limits and compiled/reference comparisons are checked too.
Fixed fuel tests accept 682 rules at exactly 4096 charges (including the final
false test) and reject the next rule in both traversal directions and engines.
Native tests isolate 64/65 statements and 16/17 nesting despite the independent
JSON guard, plus quoting at the remaining-byte boundary. Compiled adversarial
text and CLI cursor/fuel/output failures verify escaping and empty stdout.
