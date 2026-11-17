# Null and top value boundaries

The scalar prototype adds concrete singleton `Null` and abstract `Top`. JSON AST
nodes `null` / `top` are payload-free; source atoms are `null` / `_`, configured
in CUE grammar data through the existing identifier/atom pipeline. There is no
separate abstract null type. These unit variants do not change shared-text or
future graph ownership; Values remain Clone, not Copy.

## CUE-owned semantics

Top is unification identity for every represented value, including itself and
other abstract types. CUE tests both top positions before any domain handling,
then isolates null: null agrees only with null (or top already removed), and
other pairs fail type conflict. Only selected inputs clone; native code supplies
tags, equality and checked projections, not top/null unification policy. Neither
new value has integer endpoints; absent endpoints still do not admit numeric
unification or clipping.

Concrete null participates in scalar equality: null equals null, and differs
from other concrete scalar types. Top and all abstract values remain invalid
comparison operands even when structurally equal. CUE's shared concreteness
predicate now includes null, so `false && null` / `true || null` select a boolean,
while selected null, null on the left, and top operands reject. Existing eager
left-first child errors remain. Ordering, arithmetic and unary not do not gain
null/top coercions.

## Output and bootstrap

JSON scalar APIs/CLIs emit exact `null`. The legacy integer API rejects null via
IntegerOutput; bare top reports `non-concrete value` (IncompleteValue). Selecting
another abstract value through top preserves that value's existing incomplete
error. A bottom remains an execution error, not a first-class value. No structs,
lists, references, defaults, disjunctions or general solving are added.

The same six seeds regenerate evaluator/solver/backend Rust from fresh build-only
JSON, with committed Rust only as a drift baseline. CUE compiler/control/parser
budgets stay unchanged. Independent all-domain top identity/null conflict cases,
content ownership, scalar outputs, logical/equality behavior, actual compiled
alternate control, JSON/source/CLI errors and pinned Go CUE comparisons cover the
boundary. This is not full CUE or bootstrap closure, graph ownership or a runtime
performance claim.
