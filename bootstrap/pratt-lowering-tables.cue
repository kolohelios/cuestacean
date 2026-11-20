package bootstrap

let MinimumCalls = {parseSliceUpper: "parse_slice_upper", parseRecordValue: "parse_record_value", parseIndex: "parse_index", parseListElement: "parse_list_element", parseGroup: "parse_group", parseNot: "parse_not", parseNegation: "parse_negation"}
let SelectorCalls = {scanSelector: "scan_selector", advanceSelector: "advance_selector", selectorWhitespace: "selector_whitespace", scanSelectorString: "scan_selector_string", parseSelectorString: "parse_selector_string", parseSelectorIdentifier: "parse_selector_identifier", buildSelector: "build_selector"}
{
	aliases: {for name, _ in MinimumCalls {"\(name)": "parseMinimum"}}
	cases: {
		selectorAction: {op: "concat", parts: [
			{op: "text", value: "_frame.step()?;\n_frame."},
			{op: "select", field: "action", cases: {for name, method in SelectorCalls {"\(name)": {op: "text", value: method}}}},
			{op: "text", value: "()?;\n"},
		]}
		parseMinimum: {op: "concat", parts: [
			{op: "text", value: "_frame.step()?;\n_frame."},
			{op: "select", field: "op", cases: {for name, method in MinimumCalls {"\(name)": {op: "text", value: method}}}},
			{op: "text", value: "("}, {op: "lower", field: "minimum"}, {op: "text", value: ")?;\n"},
		]}
	}
}
