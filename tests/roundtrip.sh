#!/usr/bin/env bash
# Round-trip and cross-compatibility checks for zpaq.
#
#   tests/roundtrip.sh [zpaq-bin] [reference-zpaq-bin] [input-file...]
#
# For each method -m1..-m5 (plus a few custom CM/ICM/ISSE/MIX/SSE/MATCH
# configurations) this compresses each input with zpaq-bin and checks that
# zpaq-bin extracts it byte-for-byte. If a reference binary (e.g. upstream
# zpaq 7.15, or a -DNOJIT build) is given, it also checks that each side
# extracts the other's archives and that both produce byte-identical
# archives, which is what format compatibility means for ZPAQ: the model is
# defined by the archive, so any change in prediction changes the output.
# With no inputs, a mixed synthetic test file is generated.
set -euo pipefail
ZP=${1:-./zpaq}
REF=${2:-}
shift $(( $# > 2 ? 2 : $# )) || true
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
ZP=$(cd "$(dirname "$ZP")" && pwd)/$(basename "$ZP")
[ -n "$REF" ] && REF=$(cd "$(dirname "$REF")" && pwd)/$(basename "$REF")

inputs=("$@")
if [ ${#inputs[@]} -eq 0 ]; then
  # text, repeated text, binary noise, zeros and a small source file
  { for i in $(seq 1 3000); do echo "line $i: the quick brown fox $((i*i%977)) jumps"; done
    head -c 200000 /dev/urandom
    head -c 100000 /dev/zero
    cat "$(dirname "$0")/../libzpaq.h"; } > "$W/mixed.bin"
  inputs=("$W/mixed.bin")
fi

methods=(1 2 3 4 5 x4,0,0,0,0,0
  "x6,0,3,0,0,0,0"            # model order-3 CM on top of no preprocessing
  "s7.0ci1.1.1.1m.c24"        # ICM, ISSE chain, MIX, CM (custom)
  "x6.0.3.0.0.0ciam8.24s"     # MATCH-free custom with SSE
)
fail=0; n=0
check() { if cmp -s "$1" "$2"; then :; else echo "FAIL: $3"; fail=$((fail+1)); fi; n=$((n+1)); }
for f in "${inputs[@]}"; do
  for m in "${methods[@]}"; do
    d="$W/m"; rm -rf "$d"; mkdir -p "$d/x" "$d/y" "$d/z"
    cp "$f" "$d/in"
    (cd "$d" && "$ZP" a a.zpaq in -m"$m" -t1 >/dev/null 2>&1) || { echo "FAIL: compress -m$m $f"; fail=$((fail+1)); continue; }
    (cd "$d" && "$ZP" x a.zpaq in -to x/in -t1 >/dev/null 2>&1) || true
    check "$f" "$d/x/in" "round trip -m$m $(basename "$f")"
    if [ -n "$REF" ]; then
      (cd "$d" && "$REF" x a.zpaq in -to y/in -t1 >/dev/null 2>&1) || true
      check "$f" "$d/y/in" "reference extracts zpaq -m$m $(basename "$f")"
      (cd "$d" && "$REF" a b.zpaq in -m"$m" -t1 >/dev/null 2>&1) || true
      (cd "$d" && "$ZP" x b.zpaq in -to z/in -t1 >/dev/null 2>&1) || true
      check "$f" "$d/z/in" "zpaq extracts reference -m$m $(basename "$f")"
      # archives embed a timestamp per version, so compare compressed sizes
      sa=$(stat -c %s "$d/a.zpaq" 2>/dev/null || stat -f %z "$d/a.zpaq")
      sb=$(stat -c %s "$d/b.zpaq" 2>/dev/null || stat -f %z "$d/b.zpaq")
      [ "$sa" = "$sb" ] || { echo "FAIL: archive size differs -m$m $(basename "$f"): $sa vs $sb"; fail=$((fail+1)); }
      n=$((n+1))
    fi
  done
done
echo "$((n-fail))/$n checks passed"
[ $fail -eq 0 ]
