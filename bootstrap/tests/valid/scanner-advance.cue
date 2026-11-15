package bootstrap

// Structurally valid; native cursor guards must reject EOF and non-ASCII.
#ScanProgram & {body: [{op: "advance"}]}
