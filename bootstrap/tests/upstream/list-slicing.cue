import "encoding/json"

let values = [-9223372036854775808, 0, 9223372036854775807]
let mixed = [99, close({"é🙂": [true, null, "尾", 9223372036854775807]}), ["é", -9223372036854775808]]
json.Marshal([values[0:3], values[1:2], values[3:3], mixed[1:3]])
