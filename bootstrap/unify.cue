package bootstrap

#UnifyProgram & {
	let LL = {op: "input", input: "left", endpoint: "lower"}
	let LU = {op: "input", input: "left", endpoint: "upper"}
	let RL = {op: "input", input: "right", endpoint: "lower"}
	let RU = {op: "input", input: "right", endpoint: "upper"}
	let L = {op: "local", endpoint: "lower"}
	let U = {op: "local", endpoint: "upper"}
	let LeftBool = {op: "or", left: {op: "kindIs", input: "left", kind: "bool"}, right: {op: "kindIs", input: "left", kind: "boolean"}}
	let RightBool = {op: "or", left: {op: "kindIs", input: "right", kind: "bool"}, right: {op: "kindIs", input: "right", kind: "boolean"}}
	let LeftString = {op: "or", left: {op: "kindIs", input: "left", kind: "string"}, right: {op: "kindIs", input: "left", kind: "text"}}
	let RightString = {op: "or", left: {op: "kindIs", input: "right", kind: "string"}, right: {op: "kindIs", input: "right", kind: "text"}}
	result: {
		op: "if", condition: LeftString
		then: {
			op: "if", condition: RightString
			then: {
				op: "if", condition: {op: "kindIs", input: "left", kind: "string"}
				then: {op: "input", input: "right"}
				else: {
					op: "if", condition: {op: "kindIs", input: "right", kind: "string"}
					then: {op: "input", input: "left"}
					else: {
						op: "if", condition: {op: "same"}
						then: {op: "input", input: "left"}
						else: {op: "fail", reason: "strings"}
					}
				}
			}
			else: {op: "fail", reason: "types"}
		}
		else: {op: "if", condition: RightString, then: {op: "fail", reason: "types"}, else: Boolean}
	}
	let Boolean = {
		op: "if", condition: LeftBool
		then: {
			op: "if", condition: RightBool
			then: {
				op: "if", condition: {op: "kindIs", input: "left", kind: "bool"}
				then: {op: "input", input: "right"}
				else: {
					op: "if", condition: {op: "kindIs", input: "right", kind: "bool"}
					then: {op: "input", input: "left"}
					else: {
						op: "if", condition: {op: "same"}
						then: {op: "input", input: "left"}
						else: {op: "fail", reason: "booleans"}
					}
				}
			}
			else: {op: "fail", reason: "types"}
		}
		else: {
			op: "if", condition: RightBool
			then: {op: "fail", reason: "types"}
			else: Numeric
		}
	}
	let Numeric = {
		op: "if", condition: {op: "kindIs", input: "left", kind: "int"}
		then: {op: "input", input: "right"}
		else: {
			op: "if", condition: {op: "kindIs", input: "right", kind: "int"}
			then: {op: "input", input: "left"}
			else: {
				op: "if", condition: {op: "and", left: {op: "kindIs", input: "left", kind: "integer"}, right: {op: "kindIs", input: "right", kind: "integer"}}
				then: {
					op: "if", condition: {op: "compare", left: LL, right: RL, relation: "equal"}
					then: {op: "input", input: "left"}
					else: {op: "fail", reason: "integers"}
				}
				else: {
					op: "letBounds"
					lower: {
						op: "choose", condition: {op: "hasEndpoint", value: LL}
						then: {op: "choose", condition: {op: "and", left: {op: "hasEndpoint", value: RL}, right: {op: "compare", left: RL, right: LL, relation: "greater"}}, then: RL, else: LL}
						else: RL
					}
					upper: {
						op: "choose", condition: {op: "hasEndpoint", value: LU}
						then: {op: "choose", condition: {op: "and", left: {op: "hasEndpoint", value: RU}, right: {op: "compare", left: RU, right: LU, relation: "less"}}, then: RU, else: LU}
						else: RU
					}
					result: {
						op: "if", condition: {op: "and", left: {op: "hasEndpoint", value: L}, right: {op: "and", left: {op: "hasEndpoint", value: U}, right: {op: "compare", left: L, right: U, relation: "greater"}}}
						then: {op: "fail", reason: "bounds"}
						else: {
							op: "if", condition: {op: "kindIs", input: "left", kind: "integer"}
							then: {op: "input", input: "left"}
							else: {
								op: "if", condition: {op: "kindIs", input: "right", kind: "integer"}
								then: {op: "input", input: "right"}
								else: {op: "bounds", lower: L, upper: U}
							}
						}
					}
				}
			}
		}
	}
}
