# Sourced by pinned Bash; callers supply cue() and jq() wrappers.
# A selected DATA provider is never a late-refinement/raw-context interface.
# Callers run serially in a build-owned TMPDIR. Stable names keep argv receipts
# reproducible; mkdir fails closed if a path is already present.
slice_lower_serial=0
slice_lower_provider() {
    slice_provider=
    if [ "$#" -ne 3 ]; then
        printf '%s\n' 'slice lower provider: expected fixture, native JSON, raw owner' >&2
        return 1
    fi
    local fixture=$1 native_json=$2 raw_owner=$3 directory candidate provider value
    case "$fixture" in
        slice-control|tests/valid/slice-control-lower-one|tests/valid/slice-control-tail) ;;
        *) return 0 ;;
    esac
    slice_lower_serial=$((slice_lower_serial + 1))
    directory=${TMPDIR:?}/slice-lower.$slice_lower_serial
    mkdir -m 700 "$directory" || return 1
    candidate=$directory/candidate.json
    case "$fixture" in
        slice-control)
            candidate=$native_json
            ;;
        *)
            if ! cue export "$raw_owner" -e '#SliceLowerInput' --out json > "$candidate"; then
                printf '%s\n' 'slice lower provider: raw isolation export failed' >&2
                return 1
            fi
            ;;
    esac
    if ! jq -e -s 'length == 1 and (.[0] | type == "object" and keys == ["v"] and (.v | type == "number" and (. == 0 or . == 1)))' "$candidate" > /dev/null; then
        printf '%s\n' 'slice lower provider: expected exactly {v: integer 0 or 1}' >&2
        return 1
    fi
    case "$fixture" in
        slice-control)
            provider=$directory/provider.cue
            if ! cue import -p bootstrap -l '#SliceLowerInput:' -o "$provider" "$native_json" > /dev/null; then
                printf '%s\n' 'slice lower provider: selected import failed' >&2
                return 1
            fi
            ;;
        *) provider=$raw_owner ;;
    esac
    # JSON numeric comparison cannot distinguish a CUE int from e.g. 1e0.
    # Observe the value without adding a constraint that could refine it.
    if ! value=$(cue eval "$provider" -e '#SliceLowerInput.v' --concrete) ||
        [[ $value != 0 && $value != 1 ]]; then
        printf '%s\n' 'slice lower provider: expected a concrete CUE int' >&2
        return 1
    fi
    slice_provider=$provider
}
