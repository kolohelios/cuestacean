#!/usr/bin/env bash
# Compare fresh pinned-parent exports/artifacts with the integrated parser slice.
# This is an external two-tree proof, not a constructor dependency on old goldens.
set -euo pipefail
[[ $# = 4 ]] || { echo 'expected OLD_EXPORTS NEW_EXPORTS OLD_GENERATED NEW_GENERATED' >&2; exit 1; }
for directory in "$@"; do [[ $directory = /* && -d $directory ]]; done
export LC_ALL=C
compare() {
  local old=$1 new=$2 suffix=$3 relative
  while IFS= read -r relative; do
    cmp "$old/$relative" "$new/$relative" || return 1
  done < <(cd "$old" && find . -type f -name "*.$suffix" | sort)
  # Missing old paths fail cmp above; reject any unplanned new paths too.
  comm -13 <(cd "$old" && find . -type f -name "*.$suffix" | sort) \
    <(cd "$new" && find . -type f -name "*.$suffix" | sort)
}
expected_json=$(printf './%s.json\n' list-ellipsis list-ellipsis-close vet-grammar vet-pratt-lowering vet-primary vet-primary-high-minimum | sort)
expected_rust=$(printf './%s.rs\n' list-ellipsis list-ellipsis-close vet-primary vet-primary-high-minimum | sort)
# Capture only after checking cmp statuses: process substitution would hide them.
json=$(compare "$1" "$2" json)
rust=$(compare "$3" "$4" rs)
[[ $json = "$expected_json" && $rust = "$expected_rust" ]]
printf '%s\n' 'All preexisting JSON/Rust bytes preserved; only agreed vet artifacts added.'
