#!/usr/bin/env bash
# Format-compatibility regression test that needs no reference binary.
# tests/fixtures/up715-m*.zpaq were made by unmodified zpaq 7.15 (JIT build)
# from tests/fixtures/input.bin. Each must extract byte-for-byte, and
# recompressing input.bin with the same method must give an archive of the
# same size (ZPAQ coding is deterministic, so a model change shows up here).
#
#   tests/fixtures.sh [zpaq-bin]
set -euo pipefail
ZP=${1:-./zpaq}
ZP=$(cd "$(dirname "$ZP")" && pwd)/$(basename "$ZP")
D=$(cd "$(dirname "$0")/fixtures" && pwd)
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
fail=0; n=0
for a in "$D"/up715-m*.zpaq; do
  base=$(basename "$a" .zpaq); m=${base#up715-m}
  case "$m" in x6_*|s7_*) m=$(echo "$m" | tr _ .);; esac
  rm -rf "$W/t"; mkdir -p "$W/t/x"; cp "$D/input.bin" "$W/t/"
  n=$((n+1))
  if (cd "$W/t" && "$ZP" x "$a" input.bin -to x/input.bin -t1 >/dev/null 2>&1) &&
     cmp -s "$D/input.bin" "$W/t/x/input.bin"; then :; else
    echo "FAIL: extract $base"; fail=$((fail+1)); fi
  n=$((n+1))
  (cd "$W/t" && "$ZP" a new.zpaq input.bin -m"$m" -t1 >/dev/null 2>&1) || true
  sa=$(wc -c < "$a" | tr -d ' '); sb=$(wc -c < "$W/t/new.zpaq" 2>/dev/null | tr -d ' ' || echo 0)
  [ "$sa" = "$sb" ] || { echo "FAIL: -m$m archive size $sb, 7.15 gives $sa"; fail=$((fail+1)); }
done
echo "$((n-fail))/$n fixture checks passed"
[ $fail -eq 0 ]
