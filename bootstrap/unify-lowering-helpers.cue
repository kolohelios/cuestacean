package bootstrap

#UnifyLoweringSide: {op: "select", field: "input", cases: {
	left: {op: "text", value: "_left"}, right: {op: "text", value: "_right"}, third: {op: "text", value: "_third"}
}}
#UnifyLoweringEndpoint: {op: "identifier", field: "endpoint", upperFirst: true}
#UnifyLoweringConditional: {op: "concat", parts: [
	{op: "text", value: "if ("}, {op: "lower", field: "condition"}, {op: "text", value: ")? { "},
	{op: "lower", field: "then"}, {op: "text", value: " } else { "}, {op: "lower", field: "else"}, {op: "text", value: " }"},
]}
#UnifyLoweringRelation: {op: "identifier", field: "relation", upperFirst: true}
#UnifyLoweringL: {op: "lower", field: "left"}
#UnifyLoweringR: {op: "lower", field: "right"}
