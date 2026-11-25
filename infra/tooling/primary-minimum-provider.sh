# Sourced after pinned cue()/jq() wrappers. DATA is not a refinement interface.
# Stable build-owned paths keep receipts reproducible; existing paths fail closed.
primary_minimum_serial=0
primary_minimum_provider() {
    min_provider=
    if [ "$#" -ne 5 ]; then
        printf '%s\n' 'primary minimum provider: expected fixture, two JSONs, two raw owners' >&2
        return 1
    fi
    local fixture=$1 record_json=$2 conjunction_json=$3 record_raw=$4 conjunction_raw=$5
    local helper candidate raw selected directory provider value
    case "$fixture" in
        record-primary|tests/valid/record-primary-high-minimum)
            helper=RecordMinimumInput; candidate=$record_json; raw=$record_raw ;;
        conjunction-primary|tests/valid/conjunction-primary-high-minimum)
            helper=ConjunctionMinimumInput; candidate=$conjunction_json; raw=$conjunction_raw ;;
        *) return 0 ;;
    esac
    case "$fixture" in
        record-primary|conjunction-primary) selected=yes ;;
        *) selected=no ;;
    esac
    primary_minimum_serial=$((primary_minimum_serial + 1))
    directory=${TMPDIR:?}/primary-minimum.$primary_minimum_serial
    mkdir -m 700 "$directory" || return 1
    if [ "$selected" = no ]; then
        candidate=$directory/candidate.json
        if ! cue export "$raw" -e "#$helper" --out json > "$candidate"; then
            printf '%s\n' 'primary minimum provider: raw isolation export failed' >&2
            return 1
        fi
    fi
    if ! jq -e -s 'length == 1 and (.[0] | type == "object" and keys == ["v"] and (.v | type == "number" and (. == 0 or . == 100)))' "$candidate" > /dev/null; then
        printf '%s\n' 'primary minimum provider: expected exactly {v: integer 0 or 100}' >&2
        return 1
    fi
    provider=$raw
    if [ "$selected" = yes ]; then
        provider=$directory/provider.cue
        if ! cue import -p bootstrap -l "#$helper:" -o "$provider" "$candidate" > /dev/null; then
            printf '%s\n' 'primary minimum provider: selected import failed' >&2
            return 1
        fi
    fi
    # Observe without constraining/refining: JSON numbers alone lose CUE int kind.
    if ! value=$(cue eval "$provider" -e "#$helper.v" --concrete) ||
        [[ $value != 0 && $value != 100 ]]; then
        printf '%s\n' 'primary minimum provider: expected a concrete CUE int' >&2
        return 1
    fi
    min_provider=$provider
}
