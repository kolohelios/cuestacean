package bootstrap

#MetaLowering & {
	aliases: nextRule: "emitStepBack"
	cases: emitStepBack: {op: "text", value: "frame.step()?; frame.previous_rule()?;\n"}
}
