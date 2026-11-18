import "encoding/json"

// Canonical expected key order; preserve exact integer endpoints.
json.Marshal([close({a: -9223372036854775808, z: 9223372036854775807})][0])
