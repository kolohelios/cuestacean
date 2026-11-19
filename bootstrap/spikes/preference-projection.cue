package projectionpolicy

#Policy & {
	body: {
		op: "if", field: "absent"
		then: {op: "return", action: "ordinary"}
		else: {
			op: "if", field: "bottom"
			then: {op: "return", action: "ordinary"}
			else: {op: "return", action: "preferred"}
		}
	}
}
