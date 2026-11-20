package bootstrap

{cases: {
	"if": #UnifyLoweringConditional
	letBounds: {op: "concat", parts: [{op: "text", value: "{ let (_lower, _upper) = (("},
		{op: "lower", field: "lower"}, {op: "text", value: ")?, ("}, {op: "lower", field: "upper"},
		{op: "text", value: ")?); "}, {op: "lower", field: "result"}, {op: "text", value: " }"}]}
	bounds: {op: "ok", value: {op: "concat", parts: [{op: "text", value: "Value::Bounds { lower: ("},
		{op: "lower", field: "lower"}, {op: "text", value: ")?, upper: ("}, {op: "lower", field: "upper"}, {op: "text", value: ")? }"}]}}
	fail: {op: "concat", parts: [{op: "text", value: "crate::unification::fail(crate::unification::Failure::"}, {
		op: "identifier", field: "reason", upperFirst: true
	}, {op: "text", value: ", &_left, &_right)"}]}
	or: {op: "ok", value: {op: "concat", parts: [{op: "text", value: "("}, #UnifyLoweringL, {op: "text", value: ")? || ("}, #UnifyLoweringR, {op: "text", value: ")?"}]}}
	and: {op: "ok", value: {op: "concat", parts: [{op: "text", value: "("}, #UnifyLoweringL, {op: "text", value: ")? && ("}, #UnifyLoweringR, {op: "text", value: ")?"}]}}
	kindIs: {op: "ok", value: {op: "concat", parts: [{op: "text", value: "crate::unification::kind_is(&"}, #UnifyLoweringSide,
		{op: "text", value: ", crate::unification::Kind::"}, {op: "identifier", field: "kind", upperFirst: true}, {op: "text", value: ")"}]}}
	hasEndpoint: {op: "ok", value: {op: "concat", parts: [{op: "text", value: "("}, {op: "lower", field: "value"}, {op: "text", value: ")?.is_some()"}]}}
	booleanIs: {op: "concat", parts: [{op: "text", value: "crate::unification::boolean_is(&"}, #UnifyLoweringSide, {op: "text", value: ", "}, {op: "boolean", field: "value"}, {op: "text", value: ")"}]}
	textCompare: {op: "concat", parts: [{op: "text", value: "crate::unification::text_compare(&_left, &_right, crate::unification::Relation::"}, #UnifyLoweringRelation, {op: "text", value: ")"}]}
	compare: {op: "concat", parts: [{op: "text", value: "crate::unification::compare(("}, #UnifyLoweringL, {op: "text", value: ")?, ("}, #UnifyLoweringR,
		{op: "text", value: ")?, crate::unification::Relation::"}, #UnifyLoweringRelation, {op: "text", value: ")"}]}
}}
