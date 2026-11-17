# Source Unicode escapes through CUE scanner control

Single-line source strings now admit four-hex-digit `\uXXXX`. BMP scalars decode
directly; high surrogates D800..DBFF require an immediately adjacent `\uDC00`..
`\uDFFF` low surrogate. Lone lows/highs and reversed/malformed pairs reject.
Hex digits are ASCII and case insensitive. This includes escaped control values,
BOM and noncharacters; raw controls/BOM still reject. [Eight-digit `\U` scalar
escapes](wide-unicode.md) are now supported too, but wide surrogate units are
never paired. Hex/octal, interpolation, hash/raw and multiline remain unsupported.
Pinned Go CUE probes confirm the selected scalar and paired-surrogate behavior.

[Composable predicates](scanner-predicates.md) now reduce prefix-control statement
sites from 63 to 57; [bounded repeat](scanner-repeat.md) reduces them to 41.
Both the original nested and composed unrolled programs are compiled baselines.
Wide scalar admission now uses 53 statements, retaining the 41-statement
four-digit-only scanner as another compiled baseline.
CUE owns digit stepping, surrogate classification/pairing, completion and exact
diagnostics. Four digit steps reject at the first nonhex byte (including quote or
EOF). Lone low surrogates reject at their first hex byte. A high surrogate with
no adjacent `\u` rejects at the following byte; a non-low second unit rejects at
its first hex byte. The scanner temporarily uses its flag for pending high
surrogates and clears it after a valid pair; only a closing quote leaves true
at successful completion. There is no new native Unicode-scanning kernel.

Native private span/stage checks remain unchanged. Conversion runs only after
CUE admission: the JSON decoder already supports four-digit UTF-16 spelling;
a checked bridge now transcodes wide scalar escapes before that decoder. It remains defensive if trusted custom drivers admit malformed
JSON. Owned UTF-8, AST construction, exact JSON output, no normalization and
arithmetic/solver behavior are unchanged. This is not complete CUE string syntax.

A compiled legacy escape-policy alternate uses the same whole-string template
but rejects Unicode, demonstrating CUE-selected admission on unchanged input.
Independent literal/offset oracles, exhaustive BMP coverage and surrogate edges,
compiled/reference scanner and parser tests, source Go CUE comparisons and the
same six artifact-free seeds/drift paths validate the boundary. All scanner,
parser and compiler budgets remain unchanged; no performance/bootstrap closure
claim is made.
