import "encoding/json"

json.Marshal(close({a: 1, b: true}) & close({b: true, a: 1}))
