#!/bin/sh
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
bin=$(mktemp "${TMPDIR:-/tmp}/iry-feature.XXXXXX")
src=$(mktemp "${TMPDIR:-/tmp}/iry-source.XXXXXX")
trap 'rm -f "$bin" "$src"' EXIT
rustc --edition=2021 "$here/main.rs" -o "$bin"
json=$(printf 'fn main() {}\n' | "$bin" --check --json -)
printf '%s' "$json" | grep -q '"functions":1'
printf 'fn main() { println!("🦀"); }\n' >"$src"
file_json=$("$bin" --json "$src")
printf '%s' "$file_json" | grep -q '"bytes":32'
set +e
printf '// TODO\n' | "$bin" --check - >/dev/null 2>&1; check_status=$?
"$bin" --unknown </dev/null >/dev/null 2>&1; unknown_status=$?
"$bin" a.rs b.rs >/dev/null 2>&1; extra_status=$?
"$bin" /tmp/no-such-is-this-rust-yet >/dev/null 2>&1; missing_status=$?
set -e
[ "$check_status" -eq 1 ] && [ "$unknown_status" -eq 2 ] && [ "$extra_status" -eq 2 ] && [ "$missing_status" -eq 2 ]
echo 'feature CLI tests: 7 checks passed'
