# Source after pinned cue()/jq() declarations. Publish only a validated tuple.
pratt_minima_serial=0
pratt_minima_provider() {
    pratt_minimum_providers=()
    if [ "$#" -ne 5 ]; then
        printf '%s\n' 'Pratt minima: expected fixture, two JSONs, two raw owners' >&2
        return 1
    fi
    local fixture=$1 selected
    case "$fixture" in
        pratt|file-pratt) selected=yes ;;
        tests/valid/pratt-no-slices|tests/valid/pratt-slice-high-minimum|tests/valid/pratt-no-selectors|tests/valid/pratt-no-index|tests/valid/pratt-index-high-minimum|tests/valid/pratt-right)
            selected=no ;;
        *) return 0 ;;
    esac
    local -a candidates=("$2" "$3") raws=("$4" "$5")
    local -a kinds=(index slice) helpers=(PrattIndexMinimumInput PrattSliceMinimumInput)
    local -a providers=()
    local directory slot candidate provider value i
    pratt_minima_serial=$((pratt_minima_serial + 1))
    directory=${TMPDIR:?}/pratt-minima.$pratt_minima_serial
    mkdir -m 700 "$directory" || return 1
    for i in 0 1; do
        slot=$directory/${kinds[$i]}
        mkdir -m 700 "$slot" || return 1
        candidate=${candidates[$i]}
        provider=${raws[$i]}
        if [ "$selected" = no ]; then
            candidate=$slot/candidate.json
            if ! cue export "$provider" -e "#${helpers[$i]}" --out json > "$candidate"; then
                printf '%s\n' 'Pratt minima: raw isolation export failed' >&2
                return 1
            fi
        fi
        if ! jq -e -s 'length == 1 and (.[0] | type == "object" and keys == ["v"] and (.v | type == "number" and (. == 0 or . == 100)))' "$candidate" > /dev/null; then
            printf '%s\n' 'Pratt minima: expected exactly {v: integer 0 or 100}' >&2
            return 1
        fi
        if [ "$selected" = yes ]; then
            provider=$slot/provider.cue
            if ! cue import -p bootstrap -l "#${helpers[$i]}:" -o "$provider" "$candidate" > /dev/null; then
                printf '%s\n' 'Pratt minima: selected import failed' >&2
                return 1
            fi
        fi
        # Observation, not a constraint capable of backfilling an incomplete slot.
        if ! value=$(cue eval "$provider" -e "#${helpers[$i]}.v" --concrete) ||
            [[ $value != 0 && $value != 100 ]]; then
            printf '%s\n' 'Pratt minima: expected a concrete CUE int' >&2
            return 1
        fi
        providers+=("$provider")
    done
    pratt_minimum_providers=("${providers[@]}")
}
