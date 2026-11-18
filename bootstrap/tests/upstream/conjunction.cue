import "encoding/json"

json.Marshal([
	and([int, >=0, <=100, 42]),
	and([_, string, "é🙂"]),
	and([bool, true]),
	and([]) & -9223372036854775808,
	and([[null, true, 9223372036854775807], [null, true, 9223372036854775807]]),
	and([close({x: [9223372036854775807, "尾"]}), close({x: [9223372036854775807, "尾"]})]),
])
