# Exclusive checked-integer bounds

Add source `>`/`<` literal and parenthesized computed thresholds alongside
unchanged `>=`/`<=`. Within the existing i64-only value domain, CUE lowers `>n`
to inclusive lower n+1 and `<n` to inclusive upper n-1. Missing endpoints remain
infinite, not extrema; singleton bounds stay incomplete until given a candidate.
No float/string/general ordered bounds or unbounded-integer representation.

AST literal kinds lowerExclusiveBound/upperExclusiveBound carry value; computed
lowerExclusiveBoundExpression/upperExclusiveBoundExpression carry operand.
Rules eagerly evaluate/read the threshold before unary lowerExclusiveBound or
upperExclusiveBound control. CUE first requires a concrete integer, then compares
against MAX/MIN, selects unit offset and endpoint placement. Extreme >MAX/<MIN
fail `exclusive bound is empty in the signed 64-bit integer domain` (EmptyBound).
They are empty only in this bounded implementation domain, not all CUE integers.
Child errors precede threshold/empty checks, before enclosing unification RHS.

Extend the existing finite control DSL with endpoint offset:{value,delta}, delta
exactly -1 or1, and unary compare over the existing left-only endpoint subset.
Native validation eagerly rejects right/local references even in unused arms,
invalid offsets and malformed fields. Native offset_endpoint performs checked
addition on Some, preserves None/infinity and reports EndpointOffsetOverflow
(`signed 64-bit integer endpoint offset overflow`) defensively. It does not
choose exclusive policy, domain, guard, direction or bound layout. Reference
unary execution still has one operand, no dummy RHS. Existing aliases stay
serialization-only. Same compiler/backend/seed set; all guards unchanged.

CUE prefix scanning consumes >/< with optional adjacent = (longest token), and
grammar metadata selects four constructor families. Existing primary/bound-group
minimum0 and selected-driver callbacks construct literal/unary nodes; native
lookup only expands its validated token whitelist, with no new parsing loop or
semantic fallback. Spaces/trivia/radix literals/computed fields follow the same
existing profile. A preserved actual inclusive-only scanner proves the lexical
change is CUE-owned; an actual compiled exclusive-guard alternate demonstrates
independent CUE endpoint direction/layout semantics.

Evidence: fixed full AST/value/error/empty/extrema and every threshold domain;
eager errors, unary validation/arity/dead arms, checked offset/null/infinity,
metadata isolation/selected drivers/source budgets, atomic CLI failure, exact
pinned upstream positive/conflict/incomplete probes. Generated JSON stays
build-only and Rust committed; six fresh artifact-free seeds/drift/profiles and
full preflight required. No bottom Value, wider solver, bootstrap closure or
measured performance claim.
