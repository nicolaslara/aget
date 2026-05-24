#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmpdir="$(mktemp -d)"
aget_home="$tmpdir/aget-home"
out_file="$tmpdir/example.md"
js_out_file="$tmpdir/js-example.md"

cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

mkdir -p "$aget_home"
export AGET_HOME="$aget_home"

run_cmd() {
  printf '\n$'
  for arg in "$@"; do
    printf ' %q' "$arg"
  done
  printf '\n'
  "$@"
}

printf 'Repo root: %s\n' "$repo_root"
printf 'AGET_HOME: %s\n' "$aget_home"

cd "$repo_root"

run_cmd cargo run --quiet -- get --help
run_cmd cargo run --quiet -- get https://example.com
run_cmd cargo run --quiet -- --envelope json get https://example.com
run_cmd cargo run --quiet -- get https://example.com --output "$out_file"
printf 'Wrote markdown to: %s\n' "$out_file"
run_cmd cargo run --quiet -- get https://quotes.toscrape.com/js/ --output "$js_out_file" --quiet --timeout 60
printf 'Wrote JS-rendered markdown to: %s\n' "$js_out_file"
run_cmd cargo run --quiet -- https://example.com
