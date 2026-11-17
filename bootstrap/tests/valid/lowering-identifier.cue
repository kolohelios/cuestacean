package bootstrap

#LoweringPolicy & {
	formatVersion: 1
	cases: {
		root: {op: "identifier", field: "name", upperFirst: true}
		identity: {op: "identifier", field: "name"}
		snake: {op: "identifier", field: "name", snake: true}
	}
}
