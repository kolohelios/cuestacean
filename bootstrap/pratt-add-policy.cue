package bootstrap

{cases: {add: *{op: "concat", parts: [{op: "text", value: "_frame.binding_add("}, {op: "lower", field: "left"},
	{op: "text", value: ", "}, {op: "lower", field: "right"}, {op: "text", value: ")?"}]} | {op: "lower", field: "left"}}}
