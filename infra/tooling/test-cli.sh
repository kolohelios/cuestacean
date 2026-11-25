#!/usr/bin/env bash
set -euo pipefail

cli=$1
cue=$2
jq=$3
for tool in "$cli" "$cue" "$jq"; do
  [[ $tool = /* && -f $tool && -r $tool && -x $tool ]]
done
"$cue" version > version
grep -Ex 'CUE language version v0\.16\.1' version

mkdir home
rust() { env -i HOME="$PWD/home" PATH=/nonexistent "$cli" "$@"; }
upstream() { env -i HOME="$PWD/home" PATH=/nonexistent "$cue" "$@"; }

success() {
  local mode=$1 expression=$2 expected=$3
  shift 3
  printf '%s\n' "$expression" > input.cue
  printf '%s\n' "$expected" > expected.json
  rust "$@" < input.cue > actual.json 2> actual.err
  upstream "$@" < input.cue > upstream.json 2> upstream.err
  test ! -s actual.err
  test ! -s upstream.err
  if [[ $mode = exact ]]; then
    cmp expected.json actual.json
    cmp expected.json upstream.json
  else
    "$jq" -S -c . actual.json > actual-normalized
    "$jq" -S -c . upstream.json > upstream-normalized
    cmp expected.json actual-normalized
    cmp expected.json upstream-normalized
  fi
}
success exact '40 + 2' '42' export input.cue
success exact 'true' 'true' export - --out json
success exact 'null' 'null' export --out=json -o - input.cue
success exact '9223372036854775807' '9223372036854775807' export -
success exact '"cuestacean"' '"cuestacean"' export input.cue
success normalized '[1, 2 + 3]' '[1,5]' export --outfile - -
success normalized 'close({"answer": 40 + 2})' '{"answer":42}' export input.cue --outfile=-
# Explicit file and stdin share the ordinary-field profile, not the -e grammar.
for input in input.cue -; do
  success normalized $'answer:40+2\nspare:[1,2]' '{"answer":42,"spare":[1,2]}' export "$input"
  success exact $'answer:9223372036854775807\nspare:0' '9223372036854775807' export -e answer "$input"
  success normalized $'true:true\nfalse:false\nnull:null' '{"false":false,"null":null,"true":true}' export "$input"
  success normalized $'a:1+2*3\ndiv:4\nmod:5\nquo:6\nrem:7' \
    '{"a":7,"div":4,"mod":5,"quo":6,"rem":7}' export "$input"
  success normalized $'//λ\r\nanswer \t:42\r\n\r\n//tail' '{"answer":42}' export "$input"
done
printf '42\n' > ./-input.cue
rust export -- -input.cue > actual.json 2> actual.err
cmp ./-input.cue actual.json
test ! -s actual.err

success exact 'close({"answer": 40 + 2, "spare": 99})' '42' export -e answer input.cue
success exact 'close({"n": 9223372036854775807})' '9223372036854775807' export - --expression n
success exact 'close({"a_1": null})' 'null' export --expression=a_1 -- input.cue
success exact 'close({"int": true})' 'true' export -e int input.cue
success exact 'close({"if": "yes"})' '"yes"' export -e if input.cue
success normalized 'close({"items": [1, 2], "spare": 3})' '[1,2]' export -e items input.cue
success normalized 'close({"item": close({"b": [1], "a": false}), "spare": 3})' \
  '{"a":false,"b":[1]}' export -e item input.cue

failure() {
  local diagnostic=$1 rc=0
  shift
  rust "$@" > actual.json 2> actual.err || rc=$?
  test "$rc" -eq 1
  test ! -s actual.json
  grep -F -- "$diagnostic" actual.err
}
for expression in 'int' '1 & 2' '1 +'; do
  printf '%s\n' "$expression" > input.cue
  failure 'cuestacean:' export input.cue
  rc=0
  upstream export input.cue > upstream.json 2> upstream.err || rc=$?
  test "$rc" -eq 1
  test ! -s upstream.json
  test -s upstream.err
done
for expression in 'package example' 'import "strings"'; do
  printf '%s\n' "$expression" > input.cue
  failure 'cuestacean:' export input.cue
done
: > input.cue
failure 'cuestacean:' export - < input.cue
printf '\377\n' > input.cue
failure 'cannot read input.cue' export input.cue
failure 'cannot read stdin' export - < input.cue
failure 'cannot read missing.cue' export missing.cue
failure 'only the export command' eval input.cue
failure 'only the export command'
failure 'exactly one input' export
failure 'exactly one input' export input.cue input.cue
failure 'explicit .cue file' export input.json
for selector in '' true false null outer.answer '"quoted"' _hidden '#Definition' 'len(items)' 'café' 1name; do
  failure '--expression requires a single ASCII field name' export "--expression=$selector" missing.cue
done
failure '--expression requires a single ASCII field name' export missing.cue -e
failure '--expression requires a single ASCII field name' export missing.cue --expression
failure '--expression may only be specified once' export -e answer --expression=spare missing.cue
failure 'exactly one input' export input.cue -- -e answer

for expression in 'close({"spare": null})' 'close({})' '42' '[1]'; do
  printf '%s\n' "$expression" > input.cue
  case $expression in
    close*) failure 'record has no field named answer' export -e answer input.cue ;;
    *) failure '--expression requires a concrete record' export -e answer input.cue ;;
  esac
  rc=0
  upstream export -e answer input.cue > upstream.json 2> upstream.err || rc=$?
  test "$rc" -eq 1
  test ! -s upstream.json
  test -s upstream.err
done

# These are deliberate limits, not upstream-compatible failures.
# Keep upstream-success exclusions separate from differential passes.
for expression in $'a\n:1' $'a:1\na:1' 'a:{b:1}' '"a":1'; do
  printf '%s\n' "$expression" > input.cue
  for input in input.cue -; do
    failure 'cuestacean:' export "$input" < input.cue
    upstream export "$input" < input.cue > upstream.json 2> upstream.err
    test -s upstream.json
    test ! -s upstream.err
  done
done
for expression in $'a:1\rb:2' $'a:1\nb:' $'a:1\n42' $'a:1\n+2'; do
  printf '%s\n' "$expression" > input.cue
  failure 'cuestacean:' export input.cue
  failure 'cuestacean:' export - < input.cue
done
printf '42\n' > expected.json
for sibling in int '1 & 2'; do
  printf 'close({"answer": 42, "sibling": %s})\n' "$sibling" > input.cue
  failure 'cuestacean:' export -e answer input.cue
  upstream export -e answer input.cue > upstream.json 2> upstream.err
  cmp expected.json upstream.json
  test ! -s upstream.err
  printf 'answer:42\nsibling:%s\n' "$sibling" > input.cue
  for input in input.cue -; do
    failure 'cuestacean:' export -e answer "$input" < input.cue
    upstream export -e answer "$input" < input.cue > upstream.json 2> upstream.err
    cmp expected.json upstream.json
    test ! -s upstream.err
  done
done
failure 'unsupported export option: --bogus' export --bogus input.cue
failure '--out requires json' export input.cue --out
failure '--out requires json' export --out yaml input.cue
failure '--out may only be specified once' export --out=json --out json input.cue
failure 'only stdout (-)' export -o forbidden.json input.cue
failure 'only stdout (-)' export input.cue -o
failure 'output destination may only be specified once' export -o - --outfile=- input.cue
test ! -e forbidden.json
