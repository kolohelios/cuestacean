package bootstrap

// Keep Euclidean division control unchanged; compile its primitive as truncating.
#UnifyLowering & {cases: integerPrimitive: cases: divide: value: "crate::runtime::integer_quotient(&_left, &_right)"}
