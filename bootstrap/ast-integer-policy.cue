package bootstrap

{cases: {
	integer: *{op: "concat", parts: [{op: "text", value: "Value::from("}, {op: "signed", field: "value"}, {op: "text", value: "i64)"}]} | {op: "text", value: _}
}}
