package bootstrap

// Vet-only overlay; not a canonical compiler constructor input.
#LoweringPolicy & {cases: {
	hasListEllipsis: {op: "text", value: "_frame.has_list_ellipsis()?"}
}}
