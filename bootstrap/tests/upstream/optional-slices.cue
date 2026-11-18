import "encoding/json"

let values = [-9223372036854775808, 0, 9223372036854775807, "é🙂"]
let mixed = [99, close({x: [-9223372036854775808, "尾"]})]
json.Marshal([values[:], values[1:], values[:2], values[1:3], ([] )[:], mixed[1:]])
