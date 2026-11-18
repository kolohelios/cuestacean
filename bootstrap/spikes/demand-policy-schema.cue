package demandpolicy

// Test-only executable dispatch DSL; native certificates guard action execution.
#Action: "constraints" | "flat" | "defer"
#Fact:   "empty" | "forced" | "recipe" | "nested"
#Decision: {
	op:     "return"
	action: #Action
} | {
	op:    "if"
	field: #Fact
	then:  #Decision
	else:  #Decision
}
#Policy: {
	formatVersion: 1
	body:          #Decision
}
