package bootstrap

// Executable scanner statements, separate from expression evaluation rules.
// Control flow lives in these programs; cursor safety remains native.
#ScanPredicate: #ScanByteIn | #ScanCharIn | #ScanFlagIs | #ScanHasByte | {
	op: "and" | "or", left: #ScanPredicate, right: #ScanPredicate
}
#ScanByteIn: {
	ahead?: uint & <=64
	op:     "byteIn"
	bytes: [uint & <=127, ...uint & <=127]
}
#ScanCharIn: {
	op: "charIn"
	characters: [string & =~"(?s)^.$", ...string & =~"(?s)^.$"]
}
#ScanFlagIs: {op: "flagIs", value: bool}
#ScanHasByte: {op: "hasByte"}

#ScanRepeat: {op: "repeat", count: uint & <=64, body: [...#ScanStatement]}
#ScanStatement: #ScanRepeat | #ScanWhile | #ScanIf | #ScanAdvance | #ScanAdvanceChar | #ScanBreak | #ScanSetFlag | #ScanReject
#ScanWhile: {
	op:        "while"
	condition: #ScanPredicate
	body: [...#ScanStatement]
}
#ScanIf: {
	op:        "if"
	condition: #ScanPredicate
	then: [...#ScanStatement]
	else: *[] | [...#ScanStatement]
}
#ScanAdvance: {op: "advance"}
#ScanAdvanceChar: {op: "advanceChar"}
#ScanBreak: {op: "break"}
#ScanSetFlag: {op: "setFlag", value: bool}
#ScanReject: {op: "reject", reason: string & !=""}
#ScanProgram: {
	formatVersion: 1
	body: [...#ScanStatement]
}
