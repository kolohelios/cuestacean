import "encoding/json"

json.Marshal(close({empty: [], items: [close({v: 9223372036854775807}), [-9223372036854775808, "é🙂"]]}))
