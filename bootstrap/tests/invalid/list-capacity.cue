package bootstrap

import "list"

#Program & {entry: {kind: "list", value: [for _ in list.Range(0, 65, 1) {0}]}}
