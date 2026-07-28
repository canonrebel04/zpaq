#!/usr/bin/env bash
# One-line installer script for ZPAQ (AVX2 SIMD Accelerated)

set -euo pipefail

PREFIX="${PREFIX:-/usr/local}"
BINDIR="${BINDIR:-${PREFIX}/bin}"
MANDIR="${MANDIR:-${PREFIX}/share/man/man1}"

echo "==> Building and installing ZPAQ (AVX2 Accelerated) to ${PREFIX}..."

make -j"$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 2)"

echo "==> Installing zpaq to ${BINDIR}..."
if [ "$(id -u)" -eq 0 ]; then
    make install PREFIX="${PREFIX}"
else
    echo "==> Root privileges required for installation into ${PREFIX}. Using sudo..."
    sudo make install PREFIX="${PREFIX}"
fi

echo "==> Installation complete!"
echo "    zpaq: $(which zpaq 2>/dev/null || echo "${BINDIR}/zpaq")"
