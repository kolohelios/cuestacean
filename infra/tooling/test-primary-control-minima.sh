#!/usr/bin/env bash
# #300 consumer proof. Constructor closures and runtime execution are Nix gates.
set -euo pipefail
[[ $# = 12 ]] || { echo 'expected CUE JQ SPEC PRATT RUSTFMT EXPORTS SOURCE PROVIDER BOUND_JSON CALL_JSON LIST_JSON OUT' >&2; exit 1; }
CUE=$1 JQ=$2 SPEC=$3 PRATT=$4 RUSTFMT=$5 EXPORTS=$6 SOURCE=$7
PROVIDER=$8 BOUND_JSON=$9 CALL_JSON=${10} LIST_JSON=${11} OUT=${12}
for tool in "$CUE" "$JQ" "$SPEC" "$PRATT" "$RUSTFMT"; do
  [[ $tool = /* && -f $tool && -r $tool && -x $tool ]]
done
for path in "$EXPORTS" "$SOURCE" "$PROVIDER" "$BOUND_JSON" "$CALL_JSON" "$LIST_JSON" "$OUT"; do [[ $path = /* ]]; done
mkdir -p "$OUT/work/home" "$OUT/work/tmp" "$OUT/receipts" "$OUT/runtime"
cd "$OUT/work"
HOME_DIR=$PWD/home
export TMPDIR=$PWD/tmp
TRACE=$OUT/receipts/cue-argv
: > "$TRACE"
cue() {
  local invocation
  printf -v invocation '%q ' "$@"
  printf '%s\n' "$invocation" >> "$TRACE"
  env -i HOME="$HOME_DIR" PATH=/nonexistent "$CUE" "$@"
}
jq() { env -i HOME="$HOME_DIR" PATH=/nonexistent "$JQ" "$@"; }
source "$PROVIDER"
declare -F primary_control_minima_provider > /dev/null
cue version > "$OUT/receipts/cue-version"
grep -Ex 'CUE language version v0\.16\.1' "$OUT/receipts/cue-version"
schema=$SOURCE/schema.cue
fragment=$SOURCE/primary-fragments.cue
slots=(bound call list)
data=("$BOUND_JSON" "$CALL_JSON" "$LIST_JSON")
raw=(); roots=(); helpers=(PrimaryBoundMinimumInput PrimaryCallMinimumInput PrimaryListMinimumInput)
for slot in "${slots[@]}"; do
  raw+=("$SOURCE/primary-$slot-minimum-input.cue")
  roots+=("$SOURCE/primary-$slot-minimum-input-root.cue")
done
keys=(primary tests/valid/primary-no-conjunction tests/valid/primary-bound-high-minimum tests/valid/primary-call-high-minimum tests/valid/primary-no-lists tests/valid/primary-no-records)
clean_json() {
  local dest=$1
  shift
  "$@" > "$dest.raw" 2> "$dest.err"
  test ! -s "$dest.err" && test -s "$dest.raw"
  jq -e -s 'length == 1' "$dest.raw" > /dev/null
  jq -S . "$dest.raw" > "$dest"
}
native() { env -i HOME="$HOME_DIR" PATH=/nonexistent "$SPEC" "$@"; }
format() { env -i HOME="$HOME_DIR" PATH=/nonexistent "$RUSTFMT" --edition 2024; }
generate() {
  env -i HOME="$HOME_DIR" PATH=/nonexistent "$PRATT" "$1" > generated.raw 2> generated.err
  test ! -s generated.err && test -s generated.raw
  format < generated.raw
}
fail() {
  local label=$1 diagnostic=$2 status=0
  shift 2
  "$@" > "$label.out" 2> "$label.err" || status=$?
  test "$status" -eq 1 && test ! -s "$label.out" && test -s "$label.err"
  grep -E "$diagnostic" "$label.err" > /dev/null
  printf '%s\t%s\t%s\n' "$label" "$status" "$diagnostic" >> "$OUT/receipts/categories"
}
eval_ok() {
  cue eval "$@" > eval.out 2> eval.err
  test -s eval.out && test ! -s eval.err
}
# Exact preexisting selector/Rust identities. Schema and guarded template may
# grow; independent whole PROGRAM/Rust expectations below pin old emitted bytes.
while read -r hash relative; do printf '%s  %s\n' "$hash" "$SOURCE/$relative"; done > unchanged.sha256 <<'HASHES'
a26cd6d0b3c728da3fc1dd912809392bafefc4cf409e50d6d37f2fae5214d913 primary.cue
ade2811802bcf80b4c4283d6adc1625ad1a93e672e76e24458d7427a2d3a0f29 tests/valid/primary-no-conjunction.cue
f03d6cfb7fbf8781b18c64053d3986937f8f710d2631fb856c55f3504da8c4eb tests/valid/primary-bound-high-minimum.cue
30e9b3187ca2c3266e0764e105dbcc1a266a77924589a3fe07009ba54d7b14d8 tests/valid/primary-call-high-minimum.cue
373b685f01028856f34872462f97d59b0c01d999fbb7fc106419c26ddaf1be0c tests/valid/primary-no-lists.cue
34b314619ac3b7375981f44e58feca46e80e43681d486fb0b415ec27ff92e43e tests/valid/primary-no-records.cue
6cc8c69ed59a335543b9f32cb2ac741cd0fd20ec953d275ee583aceb5fee1426 generated/primary.rs
52ff1527330ba84313f1a86ce13e1b0e66f413fba213317da6c121c3f82ef28b generated/primary-no-conjunction.rs
e1cc7c92d71f32de591c7d8cb7e3f24f4a2d7eb81739c87070ab83417d5c59f6 generated/primary-bound-high-minimum.rs
ab44a821eb222da573f41a8d554a138c68a08d9a01cf32031133ea51cc920dee generated/primary-call-high-minimum.rs
816ca8220423bb59acb9172d08ec02e7128acfa4680c2c8f75fa95073d75edac generated/primary-no-lists.rs
4de98ae5572788509f2587eedb8bfd6eddb7294eca075a27cf8519218e8e602b generated/primary-no-records.rs
HASHES
sha256sum -c unchanged.sha256 > "$OUT/receipts/unchanged-check"
# Preserve trailing newline while restoring the pre-provider minimum defaults.
# The guarded template is checked by independent whole PROGRAM/Rust expectations.
IFS= read -r -d '' text < "$fragment" || [[ -n $text ]]
for i in 0 1 2; do
  slot=${slots[i]}; helper=${helpers[i]}
  # Spacing is part of the original-source proof, not a formatter side effect.
  padding=' '; if [[ $slot != bound ]]; then padding='  '; fi
  needle="_${slot}Minimum:${padding}#$helper.v"
  [[ $text = *"$needle"* && ${text#*"$needle"} != *"$needle"* ]]
  text=${text/"$needle"/"_${slot}Minimum:${padding}*0 | 100"}
done
printf '%s' "$text" > original.cue
# Vet selected/default routing must import all three DATA helpers; the high
# alternate must retain all three unresolved owners, including list minimum.
for key in vet-primary vet-primary-high-minimum; do
  primary_control_minima_provider "$key" "${data[@]}" "${raw[@]}"
  [[ ${#primary_control_providers[@]} = 3 ]]
  for i in 0 1 2; do
    if [[ $key = vet-primary-high-minimum ]]; then
      [[ ${primary_control_providers[i]} = "${raw[i]}" ]]
    else
      [[ ${primary_control_providers[i]} != "${raw[i]}" ]]
      clean_json vet-helper.json cue export "${primary_control_providers[i]}" -e "#${helpers[i]}"
      jq -S . "${data[i]}" > vet-selected.json
      cmp vet-helper.json vet-selected.json
    fi
  done
done
# Hand-authored whole PROGRAM, including disabled-feature variants. No actual
# export, golden, or compiler output is used to construct these expectations.
expected_json() {
  jq -n -S --argjson b "$1" --argjson c "$2" --argjson l "$3" --arg key "$4" '
    def op($s): {op:$s};
    def ops($ss): [$ss[] | op(.)];
    def parse($s;$v): {op:$s,minimum:{op:"integer",value:$v}};
    def branch($p;$t;$e): {op:"if",condition:op($p),then:$t,else:$e};
    {formatVersion:1,target:"parser",body:(
      ops(["whitespace","scanGroup"]) +
      [branch("groupOpen";[op("advanceGroup"),parse("parseGroup";0),op("closeGroup"),op("return")];[])] +
      [op("scanNot"),branch("hasNot";[op("advanceNot"),parse("parseNot";101),op("buildNot"),op("return")];[])] +
      [op("scanNegation"),branch("hasNegation";[op("advanceNegation"),parse("parseNegation";101),op("buildNegation"),op("return")];[])] +
      [op("scanBound"),branch("hasBound";
        ops(["advanceBound","whitespace","scanGroup"]) +
        [branch("groupOpen";[op("advanceGroup"),parse("parseGroup";$b),op("closeGroup")];[op("parseBound")]),op("return")];[])] +
      [op("scanString"),branch("hasString";ops(["parseString","return"]);[])] +
      (if $key == "primary-no-lists" then [] else
        [op("scanList"),branch("hasList";
          ops(["advanceList","whitespace","probeListEnd"]) +
          [{op:"while",condition:op("hasListElement"),body:[parse("parseListElement";$l),op("appendListElement"),op("advanceListTail"),
            branch("hasListElement";ops(["whitespace","probeListEnd"]);[])]}] + ops(["buildList","return"]);[])] end) +
      [op("scanIdentifier")] +
      (if $key == "primary-no-conjunction" or $key == "primary-no-lists" or $key == "primary-no-records" then [] else [branch("hasConjunction";ops(["conjunction","return"]);[])] end) +
      (if $key == "primary-no-records" or $key == "primary-no-lists" then [] else [branch("hasRecord";ops(["record","return"]);[])] end) +
      [branch("hasCall";ops(["advanceCall","scanGroup","advanceGroup"]) + [parse("parseGroup";$c)] + ops(["closeGroup","return"]);[]),
       branch("hasIdentifier";[op("parseAtom")];[op("parseInteger")]),op("return")])}'
}
# Separate whole Rust structural expectation, not generated by interpreting the
# JSON expectation (nor by patching a committed or freshly compiled driver).
step() { printf '%s\n' '_frame.step()?;' "_frame.$1()?;"; }
open_if() { printf '%s\n' '_frame.step()?;' "if _frame.$1()? {"; }
parse_step() { printf '%s\n' '_frame.step()?;' "_frame.$1(${2}u8)?;"; }
steps() { local s; for s in "$@"; do step "$s"; done; }
expected_rust() {
  local b=$1 c=$2 l=$3 key=$4
  printf '%s\n' '// Generated by bootstrap-pratt-codegen. Do not edit by hand.' \
    'use crate::parser::{Error, Frame};' "pub fn execute(_frame: &mut Frame<'_, '_>) -> Result<(), Error> {"
  steps whitespace scan_group; open_if group_open
  step advance_group; parse_step parse_group 0; step close_group; printf '%s\n' '_frame.step()?;' 'return Ok(());' '}'
  step scan_not; open_if has_not; step advance_not; parse_step parse_not 101
  step build_not; printf '%s\n' '_frame.step()?;' 'return Ok(());' '}'
  step scan_negation; open_if has_negation; step advance_negation; parse_step parse_negation 101
  step build_negation; printf '%s\n' '_frame.step()?;' 'return Ok(());' '}'
  step scan_bound; open_if has_bound; steps advance_bound whitespace scan_group
  open_if group_open; step advance_group; parse_step parse_group "$b"; step close_group
  printf '%s\n' '} else {'; step parse_bound; printf '%s\n' '}' '_frame.step()?;' 'return Ok(());' '}'
  step scan_string; open_if has_string; step parse_string; printf '%s\n' '_frame.step()?;' 'return Ok(());' '}'
  if [[ $key != primary-no-lists ]]; then
    step scan_list; open_if has_list; steps advance_list whitespace probe_list_end
    printf '%s\n' 'while {' '_frame.step()?;' '_frame.has_list_element()?' '} {'
    parse_step parse_list_element "$l"; steps append_list_element advance_list_tail
    open_if has_list_element; steps whitespace probe_list_end; printf '%s\n' '}' '}'
    step build_list; printf '%s\n' '_frame.step()?;' 'return Ok(());' '}'
  fi
  step scan_identifier
  if [[ $key != primary-no-conjunction && $key != primary-no-lists && $key != primary-no-records ]]; then
    open_if has_conjunction; step conjunction; printf '%s\n' '_frame.step()?;' 'return Ok(());' '}'
  fi
  if [[ $key != primary-no-records && $key != primary-no-lists ]]; then
    open_if has_record; step record; printf '%s\n' '_frame.step()?;' 'return Ok(());' '}'
  fi
  open_if has_call; steps advance_call scan_group advance_group
  parse_step parse_group "$c"; step close_group; printf '%s\n' '_frame.step()?;' 'return Ok(());' '}'
  open_if has_identifier; step parse_atom; printf '%s\n' '} else {'; step parse_integer
  printf '%s\n' '}' '_frame.step()?;' 'Ok(())' '}'
}
# Select/validate the entire tuple before any composition argv is built.
compose() {
  local key=$1 selector=$2 order=$3
  shift 3
  primary_control_providers=(POISONED_PREVIOUS_SELECTION)
  primary_control_minima_provider "$key" "$@" || return
  [[ ${#primary_control_providers[@]} = 3 ]] || return 1
  local i
  for i in 0 1 2; do
    [[ ${primary_control_providers[i]} != POISONED_PREVIOUS_SELECTION ]] || return 1
    if [[ $key != primary ]]; then
      [[ ${primary_control_providers[i]} = "${@:4+i:1}" ]] || return 1
    else
      [[ ${primary_control_providers[i]} = "$TMPDIR/primary-control-minima.$primary_control_minima_serial/${slots[i]}/provider.cue" ]] || return 1
      clean_json "$TMPDIR/route-${slots[i]}.json" cue export "${primary_control_providers[i]}" -e "#${helpers[i]}" || return
      jq -S . "${@:1+i:1}" > "$TMPDIR/selected-${slots[i]}.json" || return
      cmp "$TMPDIR/route-${slots[i]}.json" "$TMPDIR/selected-${slots[i]}.json" || return
    fi
    printf '%s\t%s\t%s\t%s\n' "$key" "$order" "${slots[i]}" "${primary_control_providers[i]}" >> "$OUT/receipts/routes"
  done
  touch composition-entered
  local inputs=("$schema" "$fragment" "${primary_control_providers[@]}" "$selector")
  if [[ $order = reverse ]]; then inputs=("$selector" "${primary_control_providers[2]}" "${primary_control_providers[1]}" "${primary_control_providers[0]}" "$fragment" "$schema"); fi
  cue export "${inputs[@]}"
}
production() { compose "$1" "$2" "$3" "${data[@]}" "${raw[@]}"; }
check_program() {
  local name=$1 b=$2 c=$3 l=$4 key=${5##*/}
  expected_json "$b" "$c" "$l" "$key" > expected.json
  cmp expected.json "$name.json"
  generate "$name.json" > "$name.rs"
  expected_rust "$b" "$c" "$l" "$key" | format > expected.rs
  cmp expected.rs "$name.rs"
}
ordered() {
  inputs=("$schema" "$1" "${raw[@]}" "$2")
  if [[ $1 = "$PWD/original.cue" ]]; then inputs=("$schema" "$1" "$2"); fi
  if [[ $3 = reverse ]]; then
    local reversed=() n
    for ((n=${#inputs[@]}-1; n>=0; n--)); do reversed+=("${inputs[n]}"); done
    inputs=("${reversed[@]}")
  fi
}
for key in "${keys[@]}"; do
  name=${key##*/}; b=0 c=0 l=0
  case "$name" in primary-bound-high-minimum) b=100 ;; primary-call-high-minimum) c=100 l=100 ;; esac
  selector=$SOURCE/$key.cue
  for order in forward reverse; do
    for bridge in "$PWD/original.cue" "$fragment"; do
      ordered "$bridge" "$selector" "$order"
      eval_ok "${inputs[@]}"
      clean_json baseline.json cue export "${inputs[@]}"
      check_program baseline "$b" "$c" "$l" "$name"
      cmp baseline.rs "$SOURCE/generated/$name.rs"
    done
    clean_json routed.json production "$key" "$selector" "$order"
    check_program routed "$b" "$c" "$l" "$name"
    cmp routed.rs "$SOURCE/generated/$name.rs"
    # Canonical selector through the open route must also retain the raw tuple.
    clean_json open.json production tests/valid/primary-no-records "$selector" "$order"
    cmp routed.json open.json
  done
  jq -S . "$EXPORTS/$key.json" > published.json
  cmp routed.json published.json
done
# Preserve the existing bound override and SIMULTANEOUS call/list override as
# complete contexts: raw success above contrasts with selected-zero conflict.
for key in tests/valid/primary-bound-high-minimum tests/valid/primary-call-high-minimum; do
  for order in forward reverse; do
    fail "${key##*/}-$order" 'conflicting values|empty disjunction' production primary "$SOURCE/$key.cue" "$order"
    inputs=("$schema" "$fragment" "${primary_control_providers[@]}" "$SOURCE/$key.cue")
    if [[ $order = reverse ]]; then inputs=("$SOURCE/$key.cue" "${primary_control_providers[2]}" "${primary_control_providers[1]}" "${primary_control_providers[0]}" "$fragment" "$schema"); fi
    fail "${key##*/}-eval-$order" 'conflicting values|empty disjunction' cue eval "${inputs[@]}"
  done
done
for i in 0 1 2; do
  slot=${slots[i]}; helper=${helpers[i]}; mkdir "$slot"
  for order in forward reverse; do
    inputs=("${raw[i]}" "${roots[i]}")
    if [[ $order = reverse ]]; then inputs=("${roots[i]}" "${raw[i]}"); fi
    clean_json "$slot/native-$order.json" native "${inputs[@]}"
    clean_json helper.json cue export "${inputs[@]}"
    cmp helper.json "$slot/native-$order.json"
    jq -n -S '{v:0}' > zero.json
    cmp zero.json helper.json
    jq -S . "${data[i]}" > published.json; cmp zero.json published.json
  done
  printf 'package bootstrap\n#PrimaryControl & {_%sMinimum:(*100|0)}\n' "$slot" > "$slot/opposite.cue"
  printf 'package bootstrap\n#PrimaryControl & {_%sMinimum:100}\n' "$slot" > "$slot/high.cue"
  printf 'package bootstrap\n{l:(#PrimaryControl & {_%sMinimum:100}),r:#PrimaryControl}\n' "$slot" > "$slot/mixed.cue"
  values=(0 0 0); values[i]=100
  for order in forward reverse; do
    for bridge in "$PWD/original.cue" "$fragment"; do
      ordered "$bridge" "$PWD/$slot/opposite.cue" "$order"
      eval_ok "${inputs[@]}"
      fail "$slot/opposite-$order-$(basename "$bridge")" 'incomplete value' cue export "${inputs[@]}"
      # Path is stable in the full canonical PROGRAM; require the exact leaf.
      leaves=(body.8.then.3.then.1.minimum.value body.16.then.3.minimum.value body.12.then.3.body.0.minimum.value)
      grep -F "${leaves[i]}" "$slot/opposite-$order-$(basename "$bridge").err" > /dev/null
      ordered "$bridge" "$PWD/$slot/high.cue" "$order"
      eval_ok "${inputs[@]}"
      clean_json high.json cue export "${inputs[@]}"; check_program high "${values[@]}" primary
      ordered "$bridge" "$PWD/$slot/mixed.cue" "$order"
      clean_json mixed.json cue export "${inputs[@]}"
      jq -S .l mixed.json > child.json; check_program child "${values[@]}" primary
      jq -S .r mixed.json > child.json; check_program child 0 0 0 primary
    done
    clean_json frozen.json production primary "$PWD/$slot/opposite.cue" "$order"
    check_program frozen 0 0 0 primary
    inputs=("$schema" "$fragment" "${primary_control_providers[@]}" "$PWD/$slot/opposite.cue")
    if [[ $order = reverse ]]; then inputs=("$PWD/$slot/opposite.cue" "${primary_control_providers[2]}" "${primary_control_providers[1]}" "${primary_control_providers[0]}" "$fragment" "$schema"); fi
    eval_ok "${inputs[@]}"
    clean_json mixed-routed.json production tests/valid/primary-no-records "$PWD/$slot/mixed.cue" "$order"
    cmp mixed.json mixed-routed.json
    fail "$slot/frozen-mixed-$order" 'conflicting values|empty disjunction' production primary "$PWD/$slot/mixed.cue" "$order"
    fail "$slot/frozen-high-$order" 'conflicting values|empty disjunction' production primary "$PWD/$slot/high.cue" "$order"
    inputs=("$schema" "$fragment" "${primary_control_providers[@]}" "$PWD/$slot/high.cue")
    if [[ $order = reverse ]]; then inputs=("$PWD/$slot/high.cue" "${primary_control_providers[2]}" "${primary_control_providers[1]}" "${primary_control_providers[0]}" "$fragment" "$schema"); fi
    fail "$slot/frozen-eval-$order" 'conflicting values|empty disjunction' cue eval "${inputs[@]}"
  done
  # An unresolved helper CAN be backfilled by downstream context without gate.
  printf 'package bootstrap\n#%s: {v:int}\n' "$helper" > "$slot/incomplete.cue"
  bypass=("${raw[@]}"); bypass[i]=$PWD/$slot/incomplete.cue
  clean_json backfilled.json cue export "$schema" "$fragment" "${bypass[@]}" "$PWD/$slot/high.cue"
  check_program backfilled "${values[@]}" primary
  eval_ok "$PWD/$slot/incomplete.cue" "${roots[i]}"
  fail "$slot/incomplete" 'incomplete value' cue export "$PWD/$slot/incomplete.cue" "${roots[i]}"
done
# State and trace receipts demonstrate gate failure before schema/context argv.
gate_failure() {
  local label=$1 key=$2 start status=0
  shift 2
  rm -f composition-entered
  start=$(wc -l < "$TRACE")
  compose "$key" "$SOURCE/tests/valid/primary-no-lists.cue" forward "$@" > "$label.out" 2> "$label.err" || status=$?
  test "$status" -ne 0 && test ! -s "$label.out" && test -s "$label.err"
  test ! -e composition-entered && test "${#primary_control_providers[@]}" = 0
  tail -n "+$((start + 1))" "$TRACE" > "$OUT/receipts/$label-argv"
  for forbidden in "$schema" "$fragment" "$SOURCE/tests/valid/primary-no-lists.cue"; do
    if grep -F "$forbidden" "$OUT/receipts/$label-argv"; then return 1; fi
  done
}
for i in 0 1 2; do
  slot=${slots[i]}; helper=${helpers[i]}
  for defect in missing incomplete string bool null array fraction decimal exponent extra outside negative absent top-array top-number multiple malformed; do
    case "$defect" in
      missing) json='{}'; body='{}' ;; incomplete) json='{}'; body='{v:int}' ;;
      string) json='{"v":"0"}'; body='{v:"0"}' ;; bool) json='{"v":false}'; body='{v:false}' ;;
      null) json='{"v":null}'; body='{v:null}' ;; array) json='{"v":[]}'; body='{v:[]}' ;;
      fraction) json='{"v":0.5}'; body='{v:0.5}' ;; decimal) json='{"v":100.0}'; body='{v:100.0}' ;;
      exponent) json='{"v":1e2}'; body='{v:1e2}' ;; extra) json='{"v":0,"extra":true}'; body='{v:0,extra:true}' ;;
      outside) json='{"v":1}'; body='{v:1}' ;; negative) json='{"v":-1}'; body='{v:-1}' ;;
      absent) json='null'; body= ;; top-array) json='[]'; body='[]' ;; top-number) json='0'; body='0' ;;
      multiple) json='{"v":0} {"v":100}'; body='{v:0, v:100}' ;; malformed) json='{'; body='{' ;;
    esac
    printf '%s\n' "$json" > invalid.json
    printf 'package bootstrap\n' > invalid.cue
    if [[ -n $body ]]; then printf '#%s: %b\n' "$helper" "$body" >> invalid.cue; fi
    bad_data=("${data[@]}"); bad_data[i]=$PWD/invalid.json
    bad_raw=("${raw[@]}"); bad_raw[i]=$PWD/invalid.cue
    gate_failure "$slot-selected-$defect" primary "${bad_data[@]}" "${raw[@]}"
    gate_failure "$slot-raw-$defect" tests/valid/primary-no-lists "${data[@]}" "${bad_raw[@]}"
  done
  bad_data=("${data[@]}"); bad_data[i]=/nonexistent
  bad_raw=("${raw[@]}"); bad_raw[i]=/nonexistent
  gate_failure "$slot-no-json" primary "${bad_data[@]}" "${raw[@]}"
  gate_failure "$slot-no-file" tests/valid/primary-no-lists "${data[@]}" "${bad_raw[@]}"
  bad_raw=("${raw[@]}"); bad_raw[i]=${raw[(i+1)%3]}
  gate_failure "$slot-wrong-owner" tests/valid/primary-no-lists "${data[@]}" "${bad_raw[@]}"
  # Recovery is deliberately interleaved with failures in every tuple position.
  for key in "${keys[@]}"; do
    clean_json recovery.json production "$key" "$SOURCE/primary.cue" forward
    check_program recovery 0 0 0 primary
  done
  printf 'package bootstrap\n#%s & {extra:true}\n' "$helper" > extra.cue
  clean_json selected.json production primary "$SOURCE/primary.cue" forward
  fail "$slot/closed-extra" 'field not allowed' cue export "${primary_control_providers[i]}" "$PWD/extra.cue"
done
for key in unrelated primary-no-lists primary-nested file-primary file-pratt pratt record-primary conjunction-primary slice-control tests/valid/primary-other; do
  primary_control_providers=(POISONED_PREVIOUS_SELECTION)
  primary_control_minima_provider "$key" /nonexistent /nonexistent /nonexistent /nonexistent /nonexistent /nonexistent
  test "${#primary_control_providers[@]}" = 0
done
# Genuine source-mark mutations, each with unchanged native exporter and root.
for i in 0 1 2; do
  slot=${slots[i]}; needle='*0 | 100'
  IFS= read -r -d '' text < "${raw[i]}" || [[ -n $text ]]
  [[ $text = *"$needle"* && ${text#*"$needle"} != *"$needle"* ]]
  printf '%s' "${text/"$needle"/'*100 | 0'}" > "$slot/mutated.cue"
  for order in forward reverse; do
    inputs=("$PWD/$slot/mutated.cue" "${roots[i]}")
    if [[ $order = reverse ]]; then inputs=("${roots[i]}" "$PWD/$slot/mutated.cue"); fi
    clean_json "$slot/mutated-$order.json" native "${inputs[@]}"
    clean_json oracle.json cue export "${inputs[@]}"; cmp oracle.json "$slot/mutated-$order.json"
    jq -n -S '{v:100}' > hundred.json; cmp hundred.json oracle.json
    mutated_raw=("${raw[@]}"); mutated_raw[i]=$PWD/$slot/mutated.cue
    values=(0 0 0); values[i]=100
    clean_json changed-raw.json compose tests/valid/primary-no-records "$SOURCE/primary.cue" "$order" "${data[@]}" "${mutated_raw[@]}"
    check_program changed-raw "${values[@]}" primary
  done
done
for row in base bound call list restored; do
  row_data=("${data[@]}"); values=(0 0 0)
  for i in 0 1 2; do
    if [[ $row = "${slots[i]}" ]]; then row_data[i]=$PWD/$row/mutated-forward.json; values[i]=100; fi
  done
  for order in forward reverse; do
    clean_json "$row-$order.json" compose primary "$SOURCE/primary.cue" "$order" "${row_data[@]}" "${raw[@]}"
    check_program "$row-$order" "${values[@]}" primary
    # Exercise every open selector with mutated selected artifacts: NONE may leak.
    for key in "${keys[@]:1}"; do
      b=0 c=0 l=0
      case "$key" in *bound-high-minimum) b=100 ;; *call-high-minimum) c=100 l=100 ;; esac
      clean_json sibling.json compose "$key" "$SOURCE/$key.cue" "$order" "${row_data[@]}" "${raw[@]}"
      check_program sibling "$b" "$c" "$l" "$key"
      cmp sibling.rs "$SOURCE/generated/${key##*/}.rs"
    done
  done
  cmp "$row-forward.json" "$row-reverse.json"; cmp "$row-forward.rs" "$row-reverse.rs"
  if [[ $row = restored ]]; then
    cmp base-forward.json restored-forward.json; cmp base-forward.rs restored-forward.rs
  else
    cp "$row-forward.json" "$OUT/runtime/$row.json"
    cp "$row-forward.rs" "$OUT/runtime/$row.rs"
  fi
done
cp unchanged.sha256 "$OUT/receipts/"
printf '%s\n' 'Whole PROGRAM/Rust, inverse, route, strict gate, phase, interleaving and genuine source-mutation proof passed; runtime and closures are separate checks.' > "$OUT/receipts/result"
