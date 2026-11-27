#!/usr/bin/env bash
# #303 consumer proof. Construction closures and actual runtime rows are Nix gates.
set -euo pipefail
[[ $# = 11 ]] || { echo 'expected CUE JQ SPEC PRATT RUSTFMT EXPORTS SOURCE PROVIDER INDEX_JSON SLICE_JSON OUT' >&2; exit 1; }
CUE=$1 JQ=$2 SPEC=$3 PRATT=$4 RUSTFMT=$5 EXPORTS=$6 SOURCE=$7 PROVIDER=$8 INDEX_JSON=$9 SLICE_JSON=${10} OUT=${11}
for tool in "$CUE" "$JQ" "$SPEC" "$PRATT" "$RUSTFMT"; do [[ $tool = /* && -f $tool && -r $tool && -x $tool ]]; done
for path in "$EXPORTS" "$SOURCE" "$PROVIDER" "$INDEX_JSON" "$SLICE_JSON" "$OUT"; do [[ $path = /* ]]; done
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
# shellcheck source=/dev/null
source "$PROVIDER"
declare -F pratt_minima_provider > /dev/null
cue version > "$OUT/receipts/cue-version"
grep -Ex 'CUE language version v0\.16\.1' "$OUT/receipts/cue-version"
schema=$SOURCE/schema.cue
fragment=$SOURCE/pratt-fragments.cue
slots=(index slice); helpers=(PrattIndexMinimumInput PrattSliceMinimumInput)
data=("$INDEX_JSON" "$SLICE_JSON"); raw=(); roots=()
for slot in "${slots[@]}"; do
  raw+=("$SOURCE/pratt-$slot-minimum-input.cue")
  roots+=("$SOURCE/pratt-$slot-minimum-input-root.cue")
done
keys=(pratt file-pratt tests/valid/pratt-no-slices tests/valid/pratt-slice-high-minimum tests/valid/pratt-no-selectors tests/valid/pratt-no-index tests/valid/pratt-index-high-minimum tests/valid/pratt-right)
clean_json() {
  local dest=$1
  shift
  "$@" > "$dest.raw" 2> "$dest.err" || return
  { test ! -s "$dest.err" && test -s "$dest.raw"; } || return 1
  jq -e -s 'length == 1' "$dest.raw" > /dev/null || return
  jq -S . "$dest.raw" > "$dest"
}
native() { env -i HOME="$HOME_DIR" PATH=/nonexistent "$SPEC" "$@"; }
format() { env -i HOME="$HOME_DIR" PATH=/nonexistent "$RUSTFMT" --edition 2024; }
generate() {
  local policy=()
  if [[ $2 = file-pratt ]]; then policy=("$EXPORTS/file-pratt-lowering.json"); fi
  env -i HOME="$HOME_DIR" PATH=/nonexistent "$PRATT" "$1" "${policy[@]}" > generated.raw 2> generated.err || return
  { test ! -s generated.err && test -s generated.raw; } || return 1
  format < generated.raw
}
fail() {
  local label=$1 diagnostic=$2 status=0
  shift 2
  "$@" > "$label.out" 2> "$label.err" || status=$?
  { test "$status" -eq 1 && test ! -s "$label.out" && test -s "$label.err"; } || return 1
  grep -E "$diagnostic" "$label.err" > /dev/null || return 1
  printf '%s\t%s\t%s\n' "$label" "$status" "$diagnostic" >> "$OUT/receipts/categories"
}
eval_ok() { cue eval "$@" > eval.out 2> eval.err && test -s eval.out && test ! -s eval.err; }
wrong_status() { echo 'expected diagnostic' >&2; return 2; }
leaked_output() { echo 'partial output'; echo 'expected diagnostic' >&2; return 1; }
if fail false-status 'expected diagnostic' wrong_status ||
   fail false-output 'expected diagnostic' leaked_output; then
  echo 'negative runner accepted a bad witness' >&2
  exit 1
fi
# Exact c1d47b7b89ba55c59b20e21c87747e8a22f9f5eb identities, not derived answers.
while read -r hash relative; do printf '%s  %s\n' "$hash" "$SOURCE/$relative"; done > unchanged.sha256 <<'HASHES'
0c9a161eb025e83a67e093b01e28df602f07238f8279b6456111762e909c8e85 pratt.cue
12fd6db156a23b3aec9f3225b6fa17ed4b287db2ddc724ae28a966d62c477118 file-pratt.cue
cc062a832a459f06d75f0abfebaa6f18fb910726a9c69d7c9c538de7294fa208 tests/valid/pratt-no-slices.cue
51a35963f5cde314bd0c00e909f54bbc0a3cc0c92f5506f569984bd37f3f7a5b tests/valid/pratt-slice-high-minimum.cue
3bea56d3565315dfebba54363e02253427b467d144d125a6c14f0b436fa946a6 tests/valid/pratt-no-selectors.cue
77fedb73970288a04fa937c0804af1f426a05dcc9351362508b8b4d9b3c070b9 tests/valid/pratt-no-index.cue
ce777acc9b2813c33be6f029d729642414ede4602351081dcff9695a22f2aabe tests/valid/pratt-index-high-minimum.cue
9c1c059451fe465c1acf53983b83d0e2e84a93955b802b9bc97590d2439d0bd4 tests/valid/pratt-right.cue
1d851dfe2a0c3eaef6efd2a38bd0a800e9df5aab9bd4267749959f51f55fe9b4 generated/pratt.rs
b05aa6aed86bd59d1d545d0325e9bea068b67dfc8c020a0a9037708830653501 generated/file-pratt.rs
cce4de554c380fe8f05781325156f10ffad60f39be15353f068ed5e5a7284f8a generated/pratt-no-slices.rs
a896e1f00e5a8fb79e1df13d62576ff12ec6ff9328289209ce17810e19b929bb generated/pratt-slice-high-minimum.rs
42c44f8623a7b4f0e29d7c4a8f4b47bbf7c0e52338e19892e8b6f67f1fe815f2 generated/pratt-no-selectors.rs
ef1bce3357ac4230aefce350f89084fdc4c3334640cbe25a5db43560b413d81c generated/pratt-no-index.rs
70917b674935a5ab17dbfa85287821abc84eb2d1c3660ed30351417887cbc7d3 generated/pratt-index-high-minimum.rs
3fe0f13fff480ca1b5dfe6cfc5e5d4e7a501027a0e700bda38251728a955dd53 generated/pratt-right.rs
HASHES
sha256sum -c unchanged.sha256 > "$OUT/receipts/unchanged-check"
# Schema may grow; retain the anchor's entire 256-file emitted-byte proof.
# Exclude ONLY the six separately checked vet parser additions, not old siblings.
printf './%s.json\n' list-ellipsis list-ellipsis-close vet-grammar vet-pratt-lowering vet-primary vet-primary-high-minimum > vet-additions
while IFS= read -r file; do test -f "$EXPORTS/$file"; done < vet-additions
test "$(find "$EXPORTS" -type f | wc -l)" -eq 262
additions=$PWD/vet-additions
(cd "$EXPORTS"; find . -type f -print | grep -vxF -f "$additions" | LC_ALL=C sort | while IFS= read -r file; do sha256sum "$file"; done) > "$OUT/receipts/bundle.sha256"
printf '%s  %s\n' 3c2f78d5701a78c77886b65cf8558deb7cba2312d29dfa461091c7abd4e24097 "$OUT/receipts/bundle.sha256" | sha256sum -c - > "$OUT/receipts/bundle-check"
IFS= read -r -d '' text < "$fragment" || [[ -n $text ]]
for i in 0 1; do
  needle="_${slots[i]}Minimum:  #${helpers[i]}.v"
  [[ $text = *"$needle"* && ${text#*"$needle"} != *"$needle"* ]]
  text=${text/"$needle"/"_${slots[i]}Minimum:  *0 | 100"}
done
printf '%s' "$text" > original.cue
printf '%s  original.cue\n' 641abcc99fec492a7cfe8ff1b91cab071173aa663e0737279af29b61ace1acb5 | sha256sum -c - > "$OUT/receipts/inverse"
# Whole structural expectations; no export/compiler/golden supplies their bodies.
expected_json() {
  jq -n -S --argjson i "$1" --argjson s "$2" --arg key "$3" '
    def op($s): {op:$s}; def ops($ss): [$ss[] | op(.)];
    def branch($p;$t;$e): {op:"if",condition:op($p),then:$t,else:$e};
    def parse($s;$v): {op:$s,minimum:{op:"integer",value:$v}};
    def after: if $key == "file-pratt" then [op("fileAfterOperand"),branch("fileBoundary";[op("return")];[])] else [op("afterOperand")] end;
    def probe: branch("hasIndex";[];[op("scanSelector")]);
    def index: [op("advanceIndex"),parse("parseIndex";$i)] +
      (if $key == "pratt-no-slices" or $key == "pratt-no-selectors" then [op("closeIndex")] else
        [op("scanSlice"),branch("hasSlice";[op("advanceSlice"),parse("parseSliceUpper";$s),op("closeSlice")];[op("closeIndex")])] end);
    def selector: ops(["advanceSelector","selectorWhitespace","scanSelectorString"]) +
      [branch("hasSelectorString";[op("parseSelectorString")];[op("parseSelectorIdentifier")]),op("buildSelector")];
    {formatVersion:1,target:"parser",body:([op("primary")] + after +
      (if $key == "pratt-no-index" then [] else [op("scanIndex")] +
        (if $key == "pratt-no-selectors" then [] else [probe] end) +
        [{op:"while",condition:op(if $key == "pratt-no-selectors" then "hasIndex" else "hasPostfix" end),body:
          (if $key == "pratt-no-selectors" then index else [branch("hasIndex";index;selector)] end) + after +
          [op("scanIndex")] + (if $key == "pratt-no-selectors" then [] else [probe] end)}] end) +
      [op("lookupOperator"),{op:"while",condition:{op:"and",left:op("hasOperator"),right:{op:"atLeast",left:op("precedence"),right:op("minimum")}},body:
        [{op:"setBindingPower",value:op("precedence")}] +
        (if $key == "pratt-right" then [] else [{op:"if",condition:op("leftAssociative"),else:[],then:[{op:"setBindingPower",value:{op:"add",left:op("precedence"),right:{op:"integer",value:1}}}]}] end) +
        [op("advanceOperator"),{op:"if",condition:{op:"atLeast",left:op("tokenLength"),right:{op:"integer",value:3}},else:[],then:[op("afterWordOperator")]}] +
        ops(["parseRight","buildBinary"]) + after + [op("lookupOperator")]}])}'
}
step() { printf '%s\n' '_frame.step()?;' "_frame.$1()?;"; }
steps() { local s; for s in "$@"; do step "$s"; done; }
open_if() { printf '%s\n' '_frame.step()?;' "if _frame.$1()? {"; }
parse_step() { printf '%s\n' '_frame.step()?;' "_frame.$1(${2}u8)?;"; }
# Separate Rust expectation, not a JSON interpreter or patched generated driver.
after_rust() {
  if [[ $1 = file-pratt ]]; then
    step file_after_operand; open_if file_boundary
    printf '%s\n' '_frame.step()?;' 'return Ok(());' '}'
  else step after_operand; fi
}
probe_rust() { open_if has_index; printf '%s\n' '} else {'; step scan_selector; printf '%s\n' '}'; }
index_rust() {
  step advance_index; parse_step parse_index "$1"
  if [[ $3 = pratt-no-slices || $3 = pratt-no-selectors ]]; then step close_index; else
    step scan_slice; open_if has_slice; step advance_slice; parse_step parse_slice_upper "$2"; step close_slice
    printf '%s\n' '} else {'; step close_index; printf '%s\n' '}'
  fi
}
expected_rust() {
  local i=$1 s=$2 key=$3 condition=has_postfix
  printf '%s\n' '// Generated by bootstrap-pratt-codegen. Do not edit by hand.' \
    'use crate::parser::{Error, Frame};' "pub fn execute(_frame: &mut Frame<'_, '_>) -> Result<(), Error> {"
  step primary; after_rust "$key"
  if [[ $key != pratt-no-index ]]; then
    step scan_index
    if [[ $key = pratt-no-selectors ]]; then condition=has_index; else probe_rust; fi
    printf '%s\n' 'while {' '_frame.step()?;' "_frame.$condition()?" '} {'
    if [[ $key = pratt-no-selectors ]]; then index_rust "$i" "$s" "$key"; else
      open_if has_index; index_rust "$i" "$s" "$key"; printf '%s\n' '} else {'
      steps advance_selector selector_whitespace scan_selector_string; open_if has_selector_string
      step parse_selector_string; printf '%s\n' '} else {'; step parse_selector_identifier; printf '%s\n' '}'
      step build_selector; printf '%s\n' '}'
    fi
    after_rust "$key"; step scan_index
    if [[ $key != pratt-no-selectors ]]; then probe_rust; fi
    printf '%s\n' '}'
  fi
  step lookup_operator
  printf '%s\n' 'while {' '_frame.step()?;' '_frame.has_operator() && _frame.precedence()? >= _frame.minimum()' '} {' \
    '_frame.step()?;' '_frame.set_binding_power(_frame.precedence()?)?;'
  if [[ $key != pratt-right ]]; then
    open_if left_associative
    printf '%s\n' '_frame.step()?;' '_frame.set_binding_power(_frame.binding_add(_frame.precedence()?, 1u8)?)?;' '}'
  fi
  step advance_operator
  printf '%s\n' '_frame.step()?;' 'if _frame.token_length()? >= 3u8 {'
  step after_word_operator; printf '%s\n' '}'
  steps parse_right build_binary; after_rust "$key"; step lookup_operator
  printf '%s\n' '}' 'Ok(())' '}'
}
compose() {
  local key=$1 selector=$2 order=$3 i
  shift 3
  local candidates=("$1" "$2") owners=("$3" "$4")
  pratt_minimum_providers=(POISONED_PREVIOUS_SELECTION)
  pratt_minima_provider "$key" "$@" || return
  [[ ${#pratt_minimum_providers[@]} = 2 ]] || return 1
  for i in 0 1; do
    [[ ${pratt_minimum_providers[i]} != POISONED_PREVIOUS_SELECTION ]] || return 1
    if [[ $key != pratt && $key != file-pratt ]]; then
      [[ ${pratt_minimum_providers[i]} = "${owners[i]}" ]] || return 1
    else
      # Counter is owned and initialized by the sourced provider.
      # shellcheck disable=SC2154
      [[ ${pratt_minimum_providers[i]} = "$TMPDIR/pratt-minima.$pratt_minima_serial/${slots[i]}/provider.cue" ]] || return 1
      clean_json "$TMPDIR/route-$i.json" cue export "${pratt_minimum_providers[i]}" -e "#${helpers[i]}" || return
      jq -S . "${candidates[i]}" > "$TMPDIR/selected-$i.json" || return
      cmp "$TMPDIR/route-$i.json" "$TMPDIR/selected-$i.json" || return
    fi
    printf '%s\t%s\t%s\t%s\n' "$key" "$order" "${slots[i]}" "${pratt_minimum_providers[i]}" >> "$OUT/receipts/routes"
  done
  touch composition-entered
  local inputs=("$schema" "$fragment" "${pratt_minimum_providers[@]}" "$selector")
  if [[ $order = reverse ]]; then inputs=("$selector" "${pratt_minimum_providers[1]}" "${pratt_minimum_providers[0]}" "$fragment" "$schema"); fi
  cue export "${inputs[@]}"
}
production() { compose "$1" "$2" "$3" "${data[@]}" "${raw[@]}"; }
check_program() {
  local name=$1 i=$2 s=$3 key=${4##*/}
  expected_json "$i" "$s" "$key" > expected.json; cmp expected.json "$name.json"
  generate "$name.json" "$key" > "$name.rs"
  expected_rust "$i" "$s" "$key" | format > expected.rs; cmp expected.rs "$name.rs"
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
  name=${key##*/}; v=(0 0)
  case "$name" in pratt-index-high-minimum) v[0]=100 ;; pratt-slice-high-minimum) v[1]=100 ;; esac
  for order in forward reverse; do
    for bridge in "$PWD/original.cue" "$fragment"; do
      ordered "$bridge" "$SOURCE/$key.cue" "$order"; eval_ok "${inputs[@]}"
      clean_json baseline.json cue export "${inputs[@]}"; check_program baseline "${v[@]}" "$name"
      cmp baseline.rs "$SOURCE/generated/$name.rs"
    done
    clean_json routed.json production "$key" "$SOURCE/$key.cue" "$order"
    check_program routed "${v[@]}" "$name"; cmp routed.rs "$SOURCE/generated/$name.rs"
    clean_json open.json production tests/valid/pratt-no-slices "$SOURCE/$key.cue" "$order"; cmp routed.json open.json
  done
  jq -S . "$EXPORTS/$key.json" > published.json; cmp routed.json published.json
done
for i in 0 1; do
  slot=${slots[i]}; helper=${helpers[i]}; mkdir "$slot"
  for order in forward reverse; do
    inputs=("${raw[i]}" "${roots[i]}"); if [[ $order = reverse ]]; then inputs=("${roots[i]}" "${raw[i]}"); fi
    clean_json "$slot/native-$order.json" native "${inputs[@]}"
    clean_json oracle.json cue export "${inputs[@]}"; cmp oracle.json "$slot/native-$order.json"
    jq -n -S '{v:0}' > zero.json; cmp zero.json oracle.json
    jq -S . "${data[i]}" > published.json; cmp zero.json published.json
  done
  for mode in pratt file-pratt; do
    file=false; if [[ $mode = file-pratt ]]; then file=true; fi
    printf 'package bootstrap\n#PrattControl & {_file:%s,_%sMinimum:(*100|0)}\n' "$file" "$slot" > "$slot/opposite.cue"
    printf 'package bootstrap\n#PrattControl & {_file:%s,_%sMinimum:100}\n' "$file" "$slot" > "$slot/high.cue"
    printf 'package bootstrap\n{l:(#PrattControl & {_file:%s,_%sMinimum:100}),r:(#PrattControl & {_file:%s})}\n' "$file" "$slot" "$file" > "$slot/mixed.cue"
    v=(0 0); v[i]=100
    for order in forward reverse; do
      for bridge in "$PWD/original.cue" "$fragment"; do
        ordered "$bridge" "$PWD/$slot/opposite.cue" "$order"; eval_ok "${inputs[@]}"
        label=$slot/opposite-$mode-$order-$(basename "$bridge")
        fail "$label" 'incomplete value' cue export "${inputs[@]}"
        leaves=(body.4.body.0.then.1.minimum.value body.4.body.0.then.3.then.1.minimum.value)
        if [[ $mode = file-pratt ]]; then leaves=(body.5.body.0.then.1.minimum.value body.5.body.0.then.3.then.1.minimum.value); fi
        grep -F "${leaves[i]}" "$label.err" > /dev/null
        ordered "$bridge" "$PWD/$slot/high.cue" "$order"; eval_ok "${inputs[@]}"
        clean_json high.json cue export "${inputs[@]}"; check_program high "${v[@]}" "$mode"
        ordered "$bridge" "$PWD/$slot/mixed.cue" "$order"
        clean_json mixed.json cue export "${inputs[@]}"
        jq -S .l mixed.json > child.json; check_program child "${v[@]}" "$mode"
        jq -S .r mixed.json > child.json; check_program child 0 0 "$mode"
      done
      clean_json frozen.json production "$mode" "$PWD/$slot/opposite.cue" "$order"; check_program frozen 0 0 "$mode"
      inputs=("$schema" "$fragment" "${pratt_minimum_providers[@]}" "$PWD/$slot/opposite.cue")
      if [[ $order = reverse ]]; then inputs=("$PWD/$slot/opposite.cue" "${pratt_minimum_providers[1]}" "${pratt_minimum_providers[0]}" "$fragment" "$schema"); fi
      eval_ok "${inputs[@]}"
      clean_json mixed-routed.json production tests/valid/pratt-no-slices "$PWD/$slot/mixed.cue" "$order"; cmp mixed.json mixed-routed.json
      fail "$slot/frozen-mixed-$mode-$order" 'conflicting values|empty disjunction' production "$mode" "$PWD/$slot/mixed.cue" "$order"
      fail "$slot/frozen-high-$mode-$order" 'conflicting values|empty disjunction' production "$mode" "$PWD/$slot/high.cue" "$order"
      inputs=("$schema" "$fragment" "${pratt_minimum_providers[@]}" "$PWD/$slot/high.cue")
      if [[ $order = reverse ]]; then inputs=("$PWD/$slot/high.cue" "${pratt_minimum_providers[1]}" "${pratt_minimum_providers[0]}" "$fragment" "$schema"); fi
      fail "$slot/frozen-eval-$mode-$order" 'conflicting values|empty disjunction' cue eval "${inputs[@]}"
    done
  done
  # Ungated incomplete owners can be backfilled by a downstream literal override.
  printf 'package bootstrap\n#%s: {v:int}\n' "$helper" > "$slot/incomplete.cue"
  bypass=("${raw[@]}"); bypass[i]=$PWD/$slot/incomplete.cue
  clean_json backfilled.json cue export "$schema" "$fragment" "${bypass[@]}" "$SOURCE/tests/valid/pratt-$slot-high-minimum.cue"
  check_program backfilled "${v[@]}" pratt
  eval_ok "$PWD/$slot/incomplete.cue" "${roots[i]}"
  fail "$slot/incomplete" 'incomplete value' cue export "$PWD/$slot/incomplete.cue" "${roots[i]}"
done
# Independent simultaneous refinement and disabled-branch contexts, both orders.
for mode in pratt file-pratt; do
  file=false; if [[ $mode = file-pratt ]]; then file=true; fi
  for spec in 'true true true' 'false true true' 'true false true' 'true true false'; do
    read -r postfix selectors slices <<< "$spec"
    printf 'package bootstrap\n#PrattControl & {_file:%s,_postfix:%s,_selectors:%s,_slices:%s,_indexMinimum:100,_sliceMinimum:100}\n' "$file" "$postfix" "$selectors" "$slices" > combined.cue
    for order in forward reverse; do
      ordered "$PWD/original.cue" "$PWD/combined.cue" "$order"; clean_json original-context.json cue export "${inputs[@]}"
      clean_json raw-context.json production tests/valid/pratt-no-slices "$PWD/combined.cue" "$order"; cmp original-context.json raw-context.json
      generate original-context.json "$mode" > original-context.rs; generate raw-context.json "$mode" > raw-context.rs; cmp original-context.rs raw-context.rs
      fail "combined-$mode-$postfix-$selectors-$slices-$order" 'conflicting values|empty disjunction' production "$mode" "$PWD/combined.cue" "$order"
    done
  done
done
# Failure receipts prove isolation happens before any composition input is used.
gate_failure() {
  local label=$1 key=$2 start status=0
  shift 2
  rm -f composition-entered; start=$(wc -l < "$TRACE")
  compose "$key" "$SOURCE/pratt.cue" forward "$@" > "$label.out" 2> "$label.err" || status=$?
  { test "$status" -ne 0 && test ! -s "$label.out" && test -s "$label.err"; } || return 1
  { test ! -e composition-entered && test "${#pratt_minimum_providers[@]}" = 0; } || return 1
  tail -n "+$((start + 1))" "$TRACE" > "$OUT/receipts/$label-argv"
  for forbidden in "$schema" "$fragment" "$SOURCE/pratt.cue"; do
    if grep -F "$forbidden" "$OUT/receipts/$label-argv"; then return 1; fi
  done
}
if (compose() { pratt_minimum_providers=(); leaked_output; }; gate_failure false-gate-output pratt) ||
   (compose() { pratt_minimum_providers=(); touch composition-entered; echo 'expected diagnostic' >&2; return 1; }; gate_failure false-gate-marker pratt); then
  echo 'gate runner accepted output or premature composition' >&2
  exit 1
fi
for i in 0 1; do
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
      multiple) json='{"v":0} {"v":100}'; body='{v:0,v:100}' ;; malformed) json='{'; body='{' ;;
    esac
    printf '%s\n' "$json" > invalid.json; printf 'package bootstrap\n' > invalid.cue
    if [[ -n $body ]]; then printf '#%s: %b\n' "$helper" "$body" >> invalid.cue; fi
    bad_data=("${data[@]}"); bad_data[i]=$PWD/invalid.json
    bad_raw=("${raw[@]}"); bad_raw[i]=$PWD/invalid.cue
    for key in pratt file-pratt; do gate_failure "$slot-$key-$defect" "$key" "${bad_data[@]}" "${raw[@]}"; done
    for key in "${keys[@]:2}"; do gate_failure "$slot-${key##*/}-raw-$defect" "$key" "${data[@]}" "${bad_raw[@]}"; done
  done
  bad_data=("${data[@]}"); bad_data[i]=/nonexistent
  bad_raw=("${raw[@]}"); bad_raw[i]=/nonexistent
  gate_failure "$slot-no-json" pratt "${bad_data[@]}" "${raw[@]}"
  gate_failure "$slot-no-file" tests/valid/pratt-no-slices "${data[@]}" "${bad_raw[@]}"
  bad_raw=("${raw[@]}"); bad_raw[i]=${raw[1-i]}
  gate_failure "$slot-wrong-owner" tests/valid/pratt-no-slices "${data[@]}" "${bad_raw[@]}"
  for key in "${keys[@]}"; do
    clean_json recovery.json production "$key" "$SOURCE/pratt.cue" forward; check_program recovery 0 0 pratt
  done
  printf 'package bootstrap\n#%s & {extra:true}\n' "$helper" > extra.cue
  clean_json selected.json production pratt "$SOURCE/pratt.cue" forward
  fail "$slot/closed-extra" 'field not allowed' cue export "${pratt_minimum_providers[i]}" "$PWD/extra.cue"
  needle='*0 | 100'; IFS= read -r -d '' text < "${raw[i]}" || [[ -n $text ]]
  [[ $text = *"$needle"* && ${text#*"$needle"} != *"$needle"* ]]
  printf '%s' "${text/"$needle"/'*100 | 0'}" > "$slot/mutated.cue"
  for order in forward reverse; do
    inputs=("$PWD/$slot/mutated.cue" "${roots[i]}"); if [[ $order = reverse ]]; then inputs=("${roots[i]}" "$PWD/$slot/mutated.cue"); fi
    clean_json "$slot/mutated-$order.json" native "${inputs[@]}"
    clean_json oracle.json cue export "${inputs[@]}"; cmp oracle.json "$slot/mutated-$order.json"
    jq -n -S '{v:100}' > hundred.json; cmp hundred.json oracle.json
    mutated_raw=("${raw[@]}"); mutated_raw[i]=$PWD/$slot/mutated.cue; v=(0 0); v[i]=100
    for mode in pratt file-pratt; do
      clean_json changed-raw.json compose tests/valid/pratt-no-slices "$SOURCE/$mode.cue" "$order" "${data[@]}" "${mutated_raw[@]}"
      check_program changed-raw "${v[@]}" "$mode"
    done
  done
  needle='*100 | 0'; IFS= read -r -d '' text < "$slot/mutated.cue" || [[ -n $text ]]
  [[ $text = *"$needle"* && ${text#*"$needle"} != *"$needle"* ]]
  printf '%s' "${text/"$needle"/'*0 | 100'}" > "$slot/restored.cue"
  cmp "$slot/restored.cue" "${raw[i]}"
  for order in forward reverse; do
    inputs=("$PWD/$slot/restored.cue" "${roots[i]}"); if [[ $order = reverse ]]; then inputs=("${roots[i]}" "$PWD/$slot/restored.cue"); fi
    clean_json "$slot/restored-$order.json" native "${inputs[@]}"
    cmp "$slot/native-forward.json" "$slot/restored-$order.json"
  done
done
for count in 4 6; do
  args=(pratt "$INDEX_JSON" "$SLICE_JSON" "${raw[@]}" extra)
  pratt_minimum_providers=(POISONED_PREVIOUS_SELECTION)
  fail "arity-$count" 'expected|argument' pratt_minima_provider "${args[@]:0:count}"
  test "${#pratt_minimum_providers[@]}" = 0
done
for key in unrelated pratt-no-slices file-primary file-pratt-lowering primary record-primary conjunction-primary slice-control tests/valid/pratt-other; do
  pratt_minimum_providers=(POISONED_PREVIOUS_SELECTION)
  pratt_minima_provider "$key" /nonexistent /nonexistent /nonexistent /nonexistent
  test "${#pratt_minimum_providers[@]}" = 0
done
for row in base index slice restored; do
  row_data=("${data[@]}"); v=(0 0)
  for i in 0 1; do
    if [[ $row = "${slots[i]}" ]]; then row_data[i]=$PWD/$row/mutated-forward.json; v[i]=100; fi
    if [[ $row = restored ]]; then row_data[i]=$PWD/${slots[i]}/restored-forward.json; fi
  done
  if [[ $row != restored ]]; then mkdir -p "$OUT/runtime/$row"; fi
  for mode in pratt file-pratt; do
    for order in forward reverse; do
      clean_json "$row-$mode-$order.json" compose "$mode" "$SOURCE/$mode.cue" "$order" "${row_data[@]}" "${raw[@]}"
      check_program "$row-$mode-$order" "${v[@]}" "$mode"
    done
    cmp "$row-$mode-forward.json" "$row-$mode-reverse.json"; cmp "$row-$mode-forward.rs" "$row-$mode-reverse.rs"
    if [[ $row = restored ]]; then
      cmp "$OUT/runtime/base/$mode.json" "$row-$mode-forward.json"; cmp "$OUT/runtime/base/$mode.rs" "$row-$mode-forward.rs"
    fi
    if [[ $row != restored ]]; then
      for extension in json rs; do cp "$row-$mode-forward.$extension" "$OUT/runtime/$row/$mode.$extension"; done
    fi
  done
  for key in "${keys[@]:2}"; do
    values=(0 0); case "$key" in *index-high-minimum) values[0]=100 ;; *slice-high-minimum) values[1]=100 ;; esac
    for order in forward reverse; do
      clean_json sibling.json compose "$key" "$SOURCE/$key.cue" "$order" "${row_data[@]}" "${raw[@]}"
      check_program sibling "${values[@]}" "$key"; cmp sibling.rs "$SOURCE/generated/${key##*/}.rs"
    done
  done
done
printf '%s\n' 'eight routes, independent full expectations, gates, raw controls, genuine mutations and restoration passed' > "$OUT/receipts/consumer"
