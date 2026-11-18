package bootstrap

import "list"

#Program & {entry: {kind: "conjunction", operands: [for n in list.Range(0, 65, 1) {{kind: "integer", value: n}}]}}
