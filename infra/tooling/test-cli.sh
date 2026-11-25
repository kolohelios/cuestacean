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
success normalized '[1, 2 + 3]' '[1,5]' export --outfile - -
success normalized 'close({"answer": 40 + 2})' '{"answer":42}' export input.cue --outfile=-
printf '42\n' > ./-input.cue
rust export -- -input.cue > actual.json 2> actual.err
cmp ./-input.cue actual.json
test ! -s actual.err

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
for expression in 'value: 42' 'package example' 'import "strings"'; do
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
failure 'unsupported export option: -e' export -e value input.cue
failure 'unsupported export option: --bogus' export --bogus input.cue
failure '--out requires json' export input.cue --out
failure '--out requires json' export --out yaml input.cue
failure '--out may only be specified once' export --out=json --out json input.cue
failure 'only stdout (-)' export -o forbidden.json input.cue
failure 'only stdout (-)' export input.cue -o
failure 'output destination may only be specified once' export -o - --outfile=- input.cue
test ! -e forbidden.json
