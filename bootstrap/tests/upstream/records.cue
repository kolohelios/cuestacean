import "encoding/json"

// CUE preserves declaration order; spell the independent expected byte order.
json.Marshal(close({a: -9223372036854775808, empty: null, z: 9223372036854775807, "é": "🙂"}))
