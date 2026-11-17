package probe

import "encoding/json"

// Compact JSON without floating-point normalization of i64 list elements.
json.Marshal([null, true, -9223372036854775808, 9223372036854775807, "é"])
