package demandpolicy

#Policy & {
	body: {
		op: "if", field: "empty"
		then: {op: "return", action: "constraints"}
		else: {
			op: "if", field: "forced"
			then: {op: "return", action: "constraints"}
			else: {
				op: "if", field: "recipe"
				then: {
					op: "if", field: "nested"
					then: {op: "return", action: "defer"}
					else: {op: "return", action: "flat"}
				}
				else: {op: "return", action: "constraints"}
			}
		}
	}
}
