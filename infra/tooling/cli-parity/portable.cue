close({
  "site": "eu-" + "west",
  "policy": close({
    "retries": (>=1 & <=5) & 3,
    "quota": 6 * 7,
    "active": true && !false,
    "note": null,
  }),
  "services": [
    close({"name": "api", "ports": [8000, 8001, 8002][0:2]}),
    close({"name": "worker", "ports": [9000 + 1]}),
  ],
  "selected": (close({"items": ["α", "β"]})).items[1],
  "max": 9223372036854775807,
  "unicode": "café 🦀",
})
