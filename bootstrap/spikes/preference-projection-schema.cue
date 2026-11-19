package projectionpolicy

// Scope-specific fact/action vocabulary; represented branching shares the VM.
#Action: "ordinary" | "preferred" | "defer"
#Fact:   "absent" | "bottom"
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
