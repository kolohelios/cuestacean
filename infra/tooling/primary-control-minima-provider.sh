# Source after pinned cue()/jq() declarations. Selected DATA is not refinable.
# Validate the complete tuple in isolation before publishing any composition argv.
primary_control_minima_serial=0
primary_control_minima_provider() {
    primary_control_providers=()
    if [ "$#" -ne 7 ]; then
        printf '%s\n' 'primary control minima: expected fixture, three JSONs, three raw owners' >&2
        return 1
    fi
    local fixture=$1 selected
    case "$fixture" in
        primary) selected=yes ;;
        tests/valid/primary-no-conjunction|tests/valid/primary-bound-high-minimum|tests/valid/primary-call-high-minimum|tests/valid/primary-no-lists|tests/valid/primary-no-records)
            selected=no ;;
        *) return 0 ;;
    esac
    local -a candidates=("$2" "$3" "$4") raws=("$5" "$6" "$7")
    local -a kinds=(bound call list) helpers=(PrimaryBoundMinimumInput PrimaryCallMinimumInput PrimaryListMinimumInput)
    local -a providers=()
    local directory slot candidate provider value i
    primary_control_minima_serial=$((primary_control_minima_serial + 1))
    directory=${TMPDIR:?}/primary-control-minima.$primary_control_minima_serial
    mkdir -m 700 "$directory" || return 1
    for i in 0 1 2; do
        slot=$directory/${kinds[$i]}
        mkdir -m 700 "$slot" || return 1
        candidate=${candidates[$i]}
        provider=${raws[$i]}
        if [ "$selected" = no ]; then
            candidate=$slot/candidate.json
            if ! cue export "$provider" -e "#${helpers[$i]}" --out json > "$candidate"; then
                printf '%s\n' 'primary control minima: raw isolation export failed' >&2
                return 1
            fi
        fi
        if ! jq -e -s 'length == 1 and (.[0] | type == "object" and keys == ["v"] and (.v | type == "number" and (. == 0 or . == 100)))' "$candidate" > /dev/null; then
            printf '%s\n' 'primary control minima: expected exactly {v: integer 0 or 100}' >&2
            return 1
        fi
        if [ "$selected" = yes ]; then
            provider=$slot/provider.cue
            if ! cue import -p bootstrap -l "#${helpers[$i]}:" -o "$provider" "$candidate" > /dev/null; then
                printf '%s\n' 'primary control minima: selected import failed' >&2
                return 1
            fi
        fi
        # Observation only: do not constrain an incomplete or non-integer value.
        if ! value=$(cue eval "$provider" -e "#${helpers[$i]}.v" --concrete) ||
            [[ $value != 0 && $value != 100 ]]; then
            printf '%s\n' 'primary control minima: expected a concrete CUE int' >&2
            return 1
        fi
        providers+=("$provider")
    done
    primary_control_providers=("${providers[@]}")
}
