package bootstrap

import "list"

// / remains available for runtime grammar configuration, not default division.
// Word operators require a complete token; never consume a partial match.
let wordContinuationBytes = list.Concat([list.Range(48, 58, 1), list.Range(65, 91, 1), [95], list.Range(97, 123, 1)])
#FinishIntegerWord: [{
	op: "if"
	condition: {op: "byteIn", ahead: 3, bytes: wordContinuationBytes}
	then: []
	else: [{op: "advance"}, {op: "advance"}, {op: "advance"}, {op: "setFlag", value: true}]
}]

#ScanProgram & {body: [{
	op: "if"
	condition: {op: "byteIn", bytes: [33, 61]}
	then: [{op: "if", condition: {op: "byteIn", ahead: 1, bytes: [61]}, then: [
		{op: "advance"}, {op: "advance"}, {op: "setFlag", value: true},
	]}]
	else: [{
		op: "if"
		condition: {op: "byteIn", bytes: [38, 42, 43, 45, 47]}
		then: [{op: "if", condition: {op: "byteIn", bytes: [38]}
			then: [{op: "advance"}, {op: "if", condition: {op: "byteIn", bytes: [38]}, then: [{op: "advance"}]}]
			else: [{op: "advance"}]}, {op: "setFlag", value: true}]
		else: [
			{op: "if", condition: {op: "byteIn", bytes: [100]}, then: [// div
				{op: "if", condition: {op: "byteIn", ahead: 1, bytes: [105]}, then: [
					{op: "if", condition: {op: "byteIn", ahead: 2, bytes: [118]}, then: #FinishIntegerWord},
				]},
			]},
			{op: "if", condition: {op: "byteIn", bytes: [109]}, then: [// mod
				{op: "if", condition: {op: "byteIn", ahead: 1, bytes: [111]}, then: [
					{op: "if", condition: {op: "byteIn", ahead: 2, bytes: [100]}, then: #FinishIntegerWord},
				]},
			]},
			{op: "if", condition: {op: "byteIn", bytes: [113]}, then: [// quo
				{op: "if", condition: {op: "byteIn", ahead: 1, bytes: [117]}, then: [
					{op: "if", condition: {op: "byteIn", ahead: 2, bytes: [111]}, then: #FinishIntegerWord},
				]},
			]},
			{op: "if", condition: {op: "byteIn", bytes: [114]}, then: [// rem
				{op: "if", condition: {op: "byteIn", ahead: 1, bytes: [101]}, then: [
					{op: "if", condition: {op: "byteIn", ahead: 2, bytes: [109]}, then: #FinishIntegerWord},
				]},
			]},
		]
	}]}, {
	op: "if", condition: {op: "flagIs", value: false}
	then: [{op: "if", condition: {op: "byteIn", bytes: [60, 62]}, then: [
		{op: "advance"},
		{op: "if", condition: {op: "byteIn", bytes: [61]}, then: [{op: "advance"}]},
		{op: "setFlag", value: true},
	]}]
}, {
	op: "if", condition: {op: "flagIs", value: false}
	then: [{op: "if", condition: {op: "byteIn", bytes: [124]}, then: [
		{op: "if", condition: {op: "byteIn", ahead: 1, bytes: [124]}, then: [
			{op: "advance"}, {op: "advance"}, {op: "setFlag", value: true},
		]},
	]}]
}]}
