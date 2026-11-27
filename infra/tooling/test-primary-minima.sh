#!/usr/bin/env bash
# #298 source/consumer proof; Nix separately owns construction closures/runtime.
set -euo pipefail
[[ $# = 11 ]] || { echo 'expected CUE JQ SPEC PRATT RUSTFMT EXPORTS SOURCE PROVIDER RECORD_JSON CONJUNCTION_JSON OUT' >&2; exit 1; }
CUE=$1 JQ=$2 SPEC=$3 PRATT=$4 RUSTFMT=$5 EXPORTS=$6 SOURCE=$7
PROVIDER=$8 RECORD_JSON=$9 CONJUNCTION_JSON=${10} OUT=${11}
for tool in "$CUE" "$JQ" "$SPEC" "$PRATT" "$RUSTFMT"; do
  [[ $tool = /* && -f $tool && -r $tool && -x $tool ]]
done
for path in "$EXPORTS" "$SOURCE" "$PROVIDER" "$RECORD_JSON" "$CONJUNCTION_JSON" "$OUT"; do [[ $path = /* ]]; done
mkdir -p "$OUT/work/home" "$OUT/receipts" "$OUT/runtime"
cd "$OUT/work"
HOME_DIR=$PWD/home
export TMPDIR=$PWD/tmp
mkdir "$TMPDIR"
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
declare -F primary_minimum_provider > /dev/null
cue version > "$OUT/receipts/cue-version"
grep -Ex 'CUE language version v0\.16\.1' "$OUT/receipts/cue-version"
schema=$SOURCE/schema.cue
record_raw=$SOURCE/record-minimum-input.cue
conjunction_raw=$SOURCE/conjunction-minimum-input.cue
clean_json() {
  local destination=$1
  shift
  "$@" > "$destination.raw" 2> "$destination.err"
  test ! -s "$destination.err" && test -s "$destination.raw"
  jq -e -s 'length == 1' "$destination.raw" > /dev/null
  jq -S . "$destination.raw" > "$destination"
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
# Exact 23d31f4923499b312c9846b2f506b7c2d7ee50a1 baseline.
while read -r hash relative; do printf '%s  %s\n' "$hash" "$SOURCE/$relative"; done > unchanged.sha256 <<'HASHES'
8a3e950bff2dae554bdd400a62d42e9afa84bc2fe72baa09fc102345d14d25a2 record-primary.cue
d6e8b649eb014dc38324fbcb3608b7aaa9360d701dc4e37ed6b6ffff66e69a50 conjunction-primary.cue
2520b0e9a5e65ef62a8e127163f738d6a10404c22a7d6f480f041c3e258e585c tests/valid/record-primary-high-minimum.cue
306a0c93e4c235bb5ce3f8dcdac6c249193902ddb9bebd089771448953d76342 tests/valid/conjunction-primary-high-minimum.cue
0f819f16ae8c530905b219607712aa2e694f1d006b017788d8e315f1b8de38e8 generated/record-primary.rs
766d8687b6ea9eb0445726e12516e35c51281a6887766e3116e3a49b9e53eb4f generated/record-primary-high-minimum.rs
561cbfa68f4b97b84f807dc275a2bbbf522da499d4ad68ecc12ad3055de4318a generated/conjunction-primary.rs
082792fbb64ba0eb7d87c7ee76feb2de9fe0701f852f5a49c6ab93f171be4e91 generated/conjunction-primary-high-minimum.rs
HASHES
sha256sum -c unchanged.sha256 > "$OUT/receipts/unchanged-check"
# Whole PROGRAM expectations authored independently of exporter/oracle output.
for value in 0 100; do
  jq -n -S --argjson v "$value" '
    def op($s): {op:$s};
    {formatVersion:1,target:"parser",body:[
      op("advanceRecord"),op("openRecordCall"),op("recordWhitespace"),
      op("openRecordBody"),op("recordWhitespace"),op("probeRecordEnd"),
      {op:"while",condition:op("hasRecordField"),body:[
        op("requireRecordLabel"),op("parseRecordLabel"),op("recordLabelTrivia"),op("recordColon"),
        {op:"parseRecordValue",minimum:{op:"integer",value:$v}},
        op("appendRecordField"),op("advanceRecordTail"),
        {op:"if",condition:op("hasRecordField"),then:[op("recordWhitespace"),op("probeRecordEnd")],else:[]}]},
      op("closeRecordCall"),op("buildRecord")]}' > "expected-record-$value.json"
  jq -n -S --argjson v "$value" '
    def op($s): {op:$s};
    {formatVersion:1,target:"parser",body:[
      op("advanceConjunction"),op("openConjunctionCall"),op("whitespace"),
      op("openConjunctionList"),op("whitespace"),op("probeListEnd"),
      {op:"while",condition:op("hasListElement"),body:[
        {op:"parseListElement",minimum:{op:"integer",value:$v}},
        op("appendListElement"),op("advanceListTail"),
        {op:"if",condition:op("hasListElement"),then:[op("whitespace"),op("probeListEnd")],else:[]}]},
      op("closeConjunctionCall"),op("buildConjunction")]}' > "expected-conjunction-$value.json"
done
# Independent full Rust emission template (not copied from generated files).
step() { printf '%s\n' '_frame.step()?;' "_frame.$1()?;"; }
expected_rust() {
  local family=$1 value=$2 predicate
  printf '%s\n' '// Generated by bootstrap-pratt-codegen. Do not edit by hand.' \
    'use crate::parser::{Error, Frame};' "pub fn execute(_frame: &mut Frame<'_, '_>) -> Result<(), Error> {"
  if [[ $family = record ]]; then
    for op in advance_record open_record_call record_whitespace open_record_body record_whitespace probe_record_end; do step "$op"; done
    predicate=has_record_field
  else
    for op in advance_conjunction open_conjunction_call whitespace open_conjunction_list whitespace probe_list_end; do step "$op"; done
    predicate=has_list_element
  fi
  printf '%s\n' 'while {' '_frame.step()?;' "_frame.$predicate()?" '} {'
  if [[ $family = record ]]; then
    for op in require_record_label parse_record_label record_label_trivia record_colon; do step "$op"; done
    printf '%s\n' '_frame.step()?;' "_frame.parse_record_value(${value}u8)?;"
    step append_record_field; step advance_record_tail
  else
    printf '%s\n' '_frame.step()?;' "_frame.parse_list_element(${value}u8)?;"
    step append_list_element; step advance_list_tail
  fi
  printf '%s\n' '_frame.step()?;' "if _frame.$predicate()? {"
  if [[ $family = record ]]; then step record_whitespace; step probe_record_end
  else step whitespace; step probe_list_end; fi
  printf '%s\n' '}' '}'
  if [[ $family = record ]]; then step close_record_call; step build_record
  else step close_conjunction_call; step build_conjunction; fi
  printf '%s\n' 'Ok(())' '}'
}
for family in record conjunction; do
  for value in 0 100; do expected_rust "$family" "$value" | format > "expected-$family-$value.rs"; done
done
family_setup() {
  family=$1
  fragment=$SOURCE/$family-primary-fragments.cue
  owner=$SOURCE/$family-minimum-input.cue
  root=$SOURCE/$family-minimum-input-root.cue
  canonical=$SOURCE/$family-primary.cue
  high=$SOURCE/tests/valid/$family-primary-high-minimum.cue
  case "$family" in
    record) helper=RecordMinimumInput; param=_minimum; control=RecordPrimaryControl
      leaf=body.6.body.4.minimum.value; hash=903914428dbe474752d54fb8e57e99f8a1c804eab0c667073da3c23590962060 ;;
    conjunction) helper=ConjunctionMinimumInput; param=_operandMinimum; control=ConjunctionPrimary
      leaf=body.6.body.0.minimum.value; hash=6c59dc565db877e1b531bcf9910d5e072076c105d598935ebac1c0b9ad08475a ;;
  esac
}
# Calls the actual gate BEFORE constructing any composition argv. Explicit returns
# ensure failures stop even in a conditional invocation where errexit is disabled.
compose() {
  local key=$1 selector=$2 order=$3 data_r=$4 data_c=$5 raw_r=$6 raw_c=$7
  min_provider=POISONED_PREVIOUS_SELECTION
  primary_minimum_provider "$key" "$data_r" "$data_c" "$raw_r" "$raw_c" || return
  [[ -n $min_provider && $min_provider != POISONED_PREVIOUS_SELECTION ]] || return 1
  case "$key" in
    tests/valid/record-primary-high-minimum) [[ $min_provider = "$raw_r" ]] || return 1 ;;
    tests/valid/conjunction-primary-high-minimum) [[ $min_provider = "$raw_c" ]] || return 1 ;;
    record-primary|conjunction-primary) [[ $min_provider != "$raw_r" && $min_provider != "$raw_c" ]] || return 1 ;;
    *) return 1 ;;
  esac
  printf '%s\t%s\t%s\n' "$key" "$order" "$min_provider" >> "$OUT/receipts/routes"
  touch composition-entered
  local inputs=("$schema" "$fragment" "$min_provider" "$selector")
  if [[ $order = reverse ]]; then inputs=("$selector" "$min_provider" "$fragment" "$schema"); fi
  cue export "${inputs[@]}"
}
production() { compose "$1" "$2" "$3" "$RECORD_JSON" "$CONJUNCTION_JSON" "$record_raw" "$conjunction_raw"; }
check_program() {
  local name=$1 value=$2
  cmp "expected-$family-$value.json" "$name.json"
  generate "$name.json" > "$name.rs"
  cmp "expected-$family-$value.rs" "$name.rs"
  local suffix=
  if [[ $value = 100 ]]; then suffix=-high-minimum; fi
  cmp "$SOURCE/generated/$family-primary$suffix.rs" "$name.rs"
}
for family in record conjunction; do
  family_setup "$family"
  mkdir "$family"
  IFS= read -r -d '' text < "$fragment" || [[ -n $text ]]
  needle="$param: #$helper.v"
  [[ $text = *"$needle"* && ${text#*"$needle"} != *"$needle"* ]]
  printf '%s' "${text/"$needle"/"$param: *0 | 100"}" > "$family/original.cue"
  printf '%s  %s\n' "$hash" "$family/original.cue" | sha256sum -c - > "$OUT/receipts/$family-inverse"
  for order in forward reverse; do
    inputs=("$owner" "$root")
    if [[ $order = reverse ]]; then inputs=("$root" "$owner"); fi
    clean_json "$family/native-$order.json" native "${inputs[@]}"
    clean_json "$family/oracle-$order.json" cue export "${inputs[@]}"
    cmp "$family/native-$order.json" "$family/oracle-$order.json"
    printf '{"v":0}\n' | jq -S . > helper.json
    cmp helper.json "$family/native-$order.json"
  done
  if [[ $family = record ]]; then published=$RECORD_JSON; else published=$CONJUNCTION_JSON; fi
  jq -S . "$published" > published.json
  cmp helper.json published.json
  for mode in canonical high; do
    selector=${!mode}; value=0; key=$family-primary
    if [[ $mode = high ]]; then value=100; key=tests/valid/$family-primary-high-minimum; fi
    for order in forward reverse; do
      inputs=("$schema" "$PWD/$family/original.cue" "$selector")
      if [[ $order = reverse ]]; then inputs=("$selector" "$PWD/$family/original.cue" "$schema"); fi
      clean_json "$family/original-$mode-$order.json" cue export "${inputs[@]}"
      check_program "$family/original-$mode-$order" "$value"
      clean_json "$family/raw-$mode-$order.json" production "tests/valid/$family-primary-high-minimum" "$selector" "$order"
      check_program "$family/raw-$mode-$order" "$value"
      clean_json "$family/production-$mode-$order.json" production "$key" "$selector" "$order"
      check_program "$family/production-$mode-$order" "$value"
    done
    jq -S . "$EXPORTS/$key.json" > exported.json
    cmp "expected-$family-$value.json" exported.json
  done
  printf 'package bootstrap\n#%s & {%s:(*100|0)}\n' "$control" "$param" > "$family/opposite.cue"
  printf 'package bootstrap\n{l:(#%s & {%s:100}),r:#%s}\n' "$control" "$param" "$control" > "$family/mixed.cue"
  for order in forward reverse; do
    for bridge in "$PWD/$family/original.cue" "$fragment"; do
      providers=(); if [[ $bridge = "$fragment" ]]; then providers=("$owner"); fi
      inputs=("$schema" "$bridge" "${providers[@]}" "$PWD/$family/opposite.cue")
      if [[ $order = reverse ]]; then inputs=("$PWD/$family/opposite.cue" "${providers[@]}" "$bridge" "$schema"); fi
      eval_ok "${inputs[@]}"
      fail "$family/opposite-$order-$(basename "$bridge")" 'incomplete value|unresolved disjunction' cue export "${inputs[@]}"
      grep -F "$leaf" "$family/opposite-$order-$(basename "$bridge").err" > /dev/null
    done
    clean_json frozen.json production "$family-primary" "$PWD/$family/opposite.cue" "$order"
    cmp "expected-$family-0.json" frozen.json
    fail "$family/frozen-high-$order" 'conflicting values|empty disjunction' production "$family-primary" "$high" "$order"
    inputs=("$schema" "$fragment" "$min_provider" "$high")
    if [[ $order = reverse ]]; then inputs=("$high" "$min_provider" "$fragment" "$schema"); fi
    fail "$family/frozen-high-eval-$order" 'conflicting values|empty disjunction' cue eval "${inputs[@]}"
    clean_json mixed.json production "tests/valid/$family-primary-high-minimum" "$PWD/$family/mixed.cue" "$order"
    for side in l r; do
      value=0; if [[ $side = l ]]; then value=100; fi
      jq -S ".$side" mixed.json > child.json
      check_program child "$value"
    done
    inputs=("$schema" "$PWD/$family/original.cue" "$PWD/$family/mixed.cue")
    if [[ $order = reverse ]]; then inputs=("$PWD/$family/mixed.cue" "$PWD/$family/original.cue" "$schema"); fi
    clean_json original-mixed.json cue export "${inputs[@]}"
    cmp mixed.json original-mixed.json
  done
  fail "$family/no-owner" 'not found|undefined' cue export "$schema" "$fragment" "$canonical"
done
# Failure receipts prove no schema/fragment/selector call precedes gate rejection.
gate_failure() {
  local label=$1 key=$2 data_r=$3 data_c=$4 raw_r=$5 raw_c=$6 status=0 start
  rm -f composition-entered
  start=$(wc -l < "$TRACE")
  compose "$key" "$high" forward "$data_r" "$data_c" "$raw_r" "$raw_c" > "$label.out" 2> "$label.err" || status=$?
  test "$status" -ne 0 && test ! -s "$label.out" && test -s "$label.err"
  test ! -e composition-entered && test -z "$min_provider"
  tail -n "+$((start + 1))" "$TRACE" > "$OUT/receipts/$label-argv"
  for forbidden in "$schema" "$fragment" "$high"; do
    if grep -F "$forbidden" "$OUT/receipts/$label-argv"; then return 1; fi
  done
}
for family in record conjunction; do
  family_setup "$family"
  printf 'package bootstrap\n#%s: {v:int}\n' "$helper" > incomplete.cue
  eval_ok "$PWD/incomplete.cue" "$root"
  fail "$family/incomplete" 'incomplete value' cue export "$PWD/incomplete.cue" "$root"
  clean_json backfilled.json cue export "$schema" "$fragment" "$PWD/incomplete.cue" "$high"
  cmp "expected-$family-100.json" backfilled.json
  for defect in missing incomplete string bool null array fraction decimal exponent extra outside negative absent top-array top-number multiple; do
    case "$defect" in
      missing) json='{}'; body='{}' ;;
      incomplete) json='{}'; body='{v:int}' ;;
      string) json='{"v":"0"}'; body='{v:"0"}' ;;
      bool) json='{"v":false}'; body='{v:false}' ;;
      null) json='{"v":null}'; body='{v:null}' ;;
      array) json='{"v":[]}'; body='{v:[]}' ;;
      fraction) json='{"v":0.5}'; body='{v:0.5}' ;;
      decimal) json='{"v":100.0}'; body='{v:100.0}' ;;
      exponent) json='{"v":1e2}'; body='{v:1e2}' ;;
      extra) json='{"v":0,"extra":true}'; body='{v:0,extra:true}' ;;
      outside) json='{"v":1}'; body='{v:1}' ;;
      negative) json='{"v":-1}'; body='{v:-1}' ;;
      absent) json='null'; body= ;;
      top-array) json='[]'; body='[]' ;;
      top-number) json='0'; body='0' ;;
      multiple) json='{"v":0} {"v":100}'; body='{v:int}' ;;
    esac
    printf '%s\n' "$json" > invalid.json
    printf 'package bootstrap\n' > invalid.cue
    if [[ -n $body ]]; then printf '#%s: %s\n' "$helper" "$body" >> invalid.cue; fi
    dr=$RECORD_JSON dc=$CONJUNCTION_JSON rr=$record_raw rc=$conjunction_raw
    if [[ $family = record ]]; then dr=$PWD/invalid.json; rr=$PWD/invalid.cue
    else dc=$PWD/invalid.json; rc=$PWD/invalid.cue; fi
    gate_failure "$family-selected-$defect" "$family-primary" "$dr" "$dc" "$record_raw" "$conjunction_raw"
    gate_failure "$family-raw-$defect" "tests/valid/$family-primary-high-minimum" "$RECORD_JSON" "$CONJUNCTION_JSON" "$rr" "$rc"
  done
  gate_failure "$family-wrong-owner" "tests/valid/$family-primary-high-minimum" "$RECORD_JSON" "$CONJUNCTION_JSON" "$conjunction_raw" "$record_raw"
  gate_failure "$family-no-json" "$family-primary" /nonexistent /nonexistent "$record_raw" "$conjunction_raw"
  gate_failure "$family-no-file" "tests/valid/$family-primary-high-minimum" "$RECORD_JSON" "$CONJUNCTION_JSON" /nonexistent /nonexistent
  clean_json selected.json production "$family-primary" "$canonical" forward
  printf 'package bootstrap\n#%s & {extra:true}\n' "$helper" > extra.cue
  fail "$family/closed-extra" 'field not allowed' cue export "$min_provider" "$PWD/extra.cue"
done
# Unknown routes cannot retain an earlier selection, even with invalid inputs.
for key in unrelated record-primary-high-minimum conjunction-primary-high-minimum tests/valid/record-primary-other; do
  min_provider=POISONED_PREVIOUS_SELECTION
  primary_minimum_provider "$key" /nonexistent /nonexistent /nonexistent /nonexistent
  test -z "$min_provider"
done
# Cross-family recovery immediately after wrong-owner failures, not just startup.
for family in conjunction record conjunction record; do
  family_setup "$family"
  gate_failure "interleaved-$family-$(wc -l < "$TRACE")" "tests/valid/$family-primary-high-minimum" "$RECORD_JSON" "$CONJUNCTION_JSON" "$conjunction_raw" "$record_raw"
  if [[ $family = record ]]; then family_setup conjunction; else family_setup record; fi
  clean_json interleaved.json production "$family-primary" "$canonical" forward
  check_program interleaved 0
done
# Independent mutations, never hand-written changed DATA; BOTH drivers per row.
for family in record conjunction; do
  family_setup "$family"
  IFS= read -r -d '' text < "$owner" || [[ -n $text ]]
  needle='*0 | 100'
  [[ $text = *"$needle"* && ${text#*"$needle"} != *"$needle"* ]]
  printf '%s' "${text/"$needle"/'*100 | 0'}" > "$family/mutated.cue"
  for order in forward reverse; do
    inputs=("$PWD/$family/mutated.cue" "$root")
    if [[ $order = reverse ]]; then inputs=("$root" "$PWD/$family/mutated.cue"); fi
    clean_json "$family/mutated-$order.json" native "${inputs[@]}"
    clean_json mutated-oracle.json cue export "${inputs[@]}"
    cmp "$family/mutated-$order.json" mutated-oracle.json
    printf '{"v":100}\n' | jq -S . > helper.json
    cmp helper.json "$family/mutated-$order.json"
    rr=$record_raw rc=$conjunction_raw
    if [[ $family = record ]]; then rr=$PWD/$family/mutated.cue; else rc=$PWD/$family/mutated.cue; fi
    clean_json "$family/mutated-raw-$order.json" compose "tests/valid/$family-primary-high-minimum" "$canonical" "$order" "$RECORD_JSON" "$CONJUNCTION_JSON" "$rr" "$rc"
    check_program "$family/mutated-raw-$order" 100
  done
done
for row in base record-only conjunction-only restored; do
  dr=$RECORD_JSON dc=$CONJUNCTION_JSON rv=0 cv=0
  if [[ $row = record-only ]]; then dr=$PWD/record/mutated-forward.json; rv=100; fi
  if [[ $row = conjunction-only ]]; then dc=$PWD/conjunction/mutated-forward.json; cv=100; fi
  # Deliberate canonical/high cross-family interleaving catches stale selections.
  for key in record-primary tests/valid/conjunction-primary-high-minimum conjunction-primary tests/valid/record-primary-high-minimum; do
    case "$key" in *conjunction*) family_setup conjunction; value=$cv ;; *) family_setup record; value=$rv ;; esac
    selector=$canonical
    if [[ $key = tests/* ]]; then selector=$high; value=100; fi
    for order in forward reverse; do
      name=$family/$row-$(basename "$key")-$order
      clean_json "$name.json" compose "$key" "$selector" "$order" "$dr" "$dc" "$record_raw" "$conjunction_raw"
      check_program "$name" "$value"
      if [[ $key != tests/* ]]; then
        # Publish only compiler outputs actually produced in this mutation matrix.
        cp "$name.json" "$OUT/runtime/$family-$value.json"
        cp "$name.rs" "$OUT/runtime/$family-$value.rs"
      fi
    done
  done
  for family in record conjunction; do
    value=0; if [[ $row = record-only && $family = record || $row = conjunction-only && $family = conjunction ]]; then value=100; fi
    cmp "expected-$family-$value.json" "$family/$row-$family-primary-forward.json"
    if [[ $value = 0 ]]; then cmp "$family/base-$family-primary-forward.rs" "$family/$row-$family-primary-forward.rs"; fi
  done
done
cp unchanged.sha256 "$OUT/receipts/"
printf '%s\n' 'Focused whole-program, routing, phase and paired source-mutation proof passed; runtime execution and recursive closures are separate Nix checks.' > "$OUT/receipts/result"
