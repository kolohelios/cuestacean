// Standalone pinned-upstream oracles for the test-only solver IR. Not production
// parser inputs or bootstrap DSL programs; see docs/solver-spike.md.
forward: {a: b + 1, b: 2}
forwardReversed: {b: 2, a: b + 1}
diamond: {a: b + c, b: base + 1, c: base + 2, base: 2}
preferred: {choice: *1 | int}
refined: {choice: *1 | int, choice: 2}
mutual: {a: b, b: a}
self: {a: a + 1}
balanced: {a: b + 1, b: a - 1}
anchored: {a: b + 1, b: a - 1, b: 2}
alias: {a: b, b: a, a: 2}
dependent: {a: b + 1, b: *1 | int}
conflict: {a: 1, a: 2}
blocked: {a: b + 1, b: c, c: b}
noInverse: {a: 3 & (b + 1), b: _}
projectedAnchor: {a: b + 1, b: (a - 1) & (*2 | int)}
// These two are deliberately outside the prototype's admitted semantic range.
overflow: {a: 9223372036854775807 + 1}
multipleDefaults: {a: *1 | int, b: *2 | int}
