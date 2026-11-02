#!/usr/bin/env bash
# Single-threaded zpaq benchmark: archive size, compress and extract MB/s.
#   tests/bench.sh <input-file> "<label>=<zpaq-bin>" ... [-- methods...]
# Prints a markdown table. Each run uses -t1 in a fresh temp directory;
# extracted output is compared to the input (a mismatch is reported as FAIL).
set -euo pipefail
IN=$(readlink -f "$1"); shift
bins=(); while [ $# -gt 0 ] && [ "$1" != "--" ]; do bins+=("$1"); shift; done
[ "${1:-}" = "--" ] && shift
methods=("${@:-1 2 3 4 5}"); [ $# -eq 0 ] && methods=(1 2 3 4 5)
W=$(mktemp -d); trap 'rm -rf "$W"' EXIT
SZ=$(stat -c %s "$IN")
now() { date +%s.%N; }
echo "Input: $(basename "$IN") ($SZ bytes), -t1, $(uname -m), $(nproc) CPUs"
echo
echo "| method | build | archive bytes | ratio | compress MB/s | extract MB/s | ok |"
echo "|---|---|---:|---:|---:|---:|---|"
for m in "${methods[@]}"; do
  for lb in "${bins[@]}"; do
    l=${lb%%=*}; b=$(readlink -f "${lb#*=}")
    rm -rf "$W"/*; mkdir -p "$W/x"; cp "$IN" "$W/in"
    t0=$(now); (cd "$W" && "$b" a a.zpaq in -m"$m" -t1 >/dev/null 2>&1); t1=$(now)
    (cd "$W" && "$b" x a.zpaq in -to x/in -t1 >/dev/null 2>&1) || true; t2=$(now)
    a=$(stat -c %s "$W/a.zpaq")
    ok=FAIL; cmp -s "$IN" "$W/x/in" && ok=yes
    awk -v m="$m" -v l="$l" -v a="$a" -v s="$SZ" -v c="$(echo "$t1-$t0"|bc -l)" -v d="$(echo "$t2-$t1"|bc -l)" -v ok="$ok" \
      'BEGIN{printf "| -m%s | %s | %d | %.3f | %.2f | %.2f | %s |\n", m, l, a, s/a, s/1e6/c, s/1e6/d, ok}'
  done
done
