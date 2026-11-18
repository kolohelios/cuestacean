package bootstrap

#PrattProgram & {
	target: "conjunction"
	body: [for statement in #ConjunctionControl.body {statement}, {op: "return"}]
}
