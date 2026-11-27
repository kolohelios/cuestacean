package bootstrap

// Compose with grammar.cue; ordinary grammar DATA remains unchanged.
#Grammar & {typedList: {
	kind: "typedListExpression", elementsField: "elements", tailField: "tail"
}}
