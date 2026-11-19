package bootstrap

{cases: {
	and: {op: "concat", parts: [
		{op: "text", value: "("}, {op: "lower", field: "left"},
		{op: "ifTrue", field: "inverse", then: {op: "text", value: ") || ("}, else: {op: "text", value: ") && ("}},
		{op: "lower", field: "right"}, {op: "text", value: ")"},
	]}
	or: {op: "concat", parts: [
		{op: "text", value: "("}, {op: "lower", field: "left"},
		{op: "ifTrue", field: "inverse", then: {op: "text", value: ") && ("}, else: {op: "text", value: ") || ("}},
		{op: "lower", field: "right"}, {op: "text", value: ")"},
	]}
	byteIn: {op: "concat", parts: [#ScanNot, {op: "text", value: "matches!("}, {
		op: "ifZero", field: "ahead", then: {op: "text", value: "cursor.peek()"}, else: {op: "concat", parts: [
			{op: "text", value: "cursor.peek_at("}, {op: "unsigned", field: "ahead"}, {op: "text", value: ")"},
		]}
	}, {op: "text", value: ", Some("}, {op: "bytePattern", field: "bytes"}, {op: "text", value: "))"}]}
	charIn: {op: "concat", parts: [#ScanNot, {op: "text", value: "matches!(cursor.peek_char(), Some("},
		{op: "charPattern", field: "characters"}, {op: "text", value: "))"}]}
	flagIs: {op: "ifTrue", field: "inverse", then: {
		op: "ifTrue", field: "value", then: {op: "text", value: "!cursor.flag()"}, else: {op: "text", value: "cursor.flag()"}
	}, else: {op: "ifTrue", field: "value", then: {op: "text", value: "cursor.flag()"}, else: {op: "text", value: "!cursor.flag()"}}}
	hasByte: {op: "ifTrue", field: "inverse", then: {op: "text", value: "cursor.peek().is_none()"}, else: {op: "text", value: "cursor.peek().is_some()"}}
}}
