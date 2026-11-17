package bootstrap

#DecimalDigits: [48, 49, 50, 51, 52, 53, 54, 55, 56, 57]
#DecimalRun: {op: "while", condition: {op: "byteIn", bytes: #DecimalDigits}, body: [{op: "advance"}]}
#DecimalSign: [{op: "if", condition: {op: "byteIn", bytes: [45]}, then: [{op: "setFlag", value: true}, {op: "advance"}]}]
#UnsignedDecimal: [{
	op: "if", condition: {op: "byteIn", bytes: [48]}
	then: [{op: "advance"}, {
		op: "if", condition: {op: "byteIn", bytes: #DecimalDigits}
		then: [#DecimalRun, {op: "reject", reason: "leading zeroes are unsupported"}]
	}]
	else: [{
		op: "if", condition: {op: "byteIn", bytes: [49, 50, 51, 52, 53, 54, 55, 56, 57]}
		then: [#DecimalRun]
		else: [{op: "reject", reason: "expected signed decimal integer"}]
	}]
}]
#DecimalSpellingScanner: #ScanProgram & {body: [for step in #DecimalSign {step}, for step in #UnsignedDecimal {step}]}

#RadixDigitRun: {
	_digits: [uint, ...uint]
	_reason: string
	steps: [{op: "advance"}, {op: "advance"}, {
		op: "if", condition: {op: "byteIn", bytes: _digits}
		then: [{op: "while", condition: {op: "byteIn", bytes: _digits}, body: [{op: "advance"}]}]
		else: [{op: "reject", reason: _reason}]
	}]
}
#HexadecimalRun: #RadixDigitRun & {
	_digits: [48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 65, 66, 67, 68, 69, 70, 97, 98, 99, 100, 101, 102]
	_reason: "expected hexadecimal integer digit"
}
#BinaryRun: #RadixDigitRun & {_digits: [48, 49], _reason: "expected binary integer digit"}
#OctalRun: #RadixDigitRun & {_digits: [48, 49, 50, 51, 52, 53, 54, 55], _reason: "expected octal integer digit"}
#IntegerSpellingScanner: #ScanProgram & {body: [
	for step in #DecimalSign {step},
	{op: "if", condition: {op: "and", left: {op: "byteIn", bytes: [48]}, right: {op: "byteIn", ahead: 1, bytes: [120, 88]}}
		then: [for step in #HexadecimalRun.steps {step}]
		else: [{op: "if", condition: {op: "and", left: {op: "byteIn", bytes: [48]}, right: {op: "byteIn", ahead: 1, bytes: [98]}}
			then: [for step in #BinaryRun.steps {step}]
			else: [{op: "if", condition: {op: "and", left: {op: "byteIn", bytes: [48]}, right: {op: "byteIn", ahead: 1, bytes: [111]}}
				then: [for step in #OctalRun.steps {step}]
				else: [for step in #UnsignedDecimal {step}]
			}]
		}]
	},
]}
