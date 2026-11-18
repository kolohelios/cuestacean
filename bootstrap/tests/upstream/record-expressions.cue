import "encoding/json"

// Spell the independent byte-sorted object order; preserve exact i64s.
json.Marshal(close({count: len(close({x: null})), max: 9223372036854775806 + 1, min: [-9223372036854775808][0], "é": "🙂"}))
