import "encoding/json"

json.Marshal(close({x: close({a: 7})}) & close({x: close({a: 7})}))
