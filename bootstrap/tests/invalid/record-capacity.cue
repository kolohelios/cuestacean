package bootstrap

import "list"

#Program & {entry: {kind: "record", fields: [for n in list.Range(0, 65, 1) {name: "f\(n)", value: n}]}}
