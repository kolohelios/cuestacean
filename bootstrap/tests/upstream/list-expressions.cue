package probe

import "encoding/json"

json.Marshal([1+2, len("é🙂"), null, true, "é"])
