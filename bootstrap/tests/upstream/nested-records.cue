import "encoding/json"

// CUE preserves declaration order; use the independent byte-sorted object order.
json.Marshal(close({
	empty: close({}),
	limits: close({max: 9223372036854775807, min: -9223372036854775808}),
	selected: close({child: close({x: 2+3})})["child"],
	text: "owned 🙂",
}))
