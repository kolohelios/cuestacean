package bootstrap

// Selected only for the two file-profile control programs, never a compiler
// constructor input or fallback for canonical Pratt policy generation.
#LoweringPolicy & {cases: {
	fileBoundary: {op: "text", value: "_frame.file_boundary()?"}
	hasFileField: {op: "text", value: "_frame.has_file_field()?"}
}}
