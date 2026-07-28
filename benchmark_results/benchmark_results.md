# ZPAQ Compression Engine Benchmark Report

**Date:** 2026-07-28 01:41:59
**Target System:** Linux (Kernel: `7.1.4-1-cachyos`, Architecture: `x86_64`)

## Executive Summary

This report documents the performance, compression efficiency, memory consumption, and thread scaling characteristics of the ZPAQ journaling compression engine.

### Summary Matrix by Data Type

| Corpus | Type | Size (MB) | L1 Ratio | L5 Ratio | L1 Comp Speed | L5 Comp Speed | L1 Decomp Speed | Max RAM (MB) | Verification |
|---|---|---|---|---|---|---|---|---|---|
| `text` | Text | 2.00 | 92.30% (12.98x) | N/A | 75.55 MB/s | N/A | 267.64 MB/s | 14.3 MB | ✅ PASS |
| `binary` | Binary | 2.00 | 82.17% (5.61x) | N/A | 52.63 MB/s | N/A | 166.01 MB/s | 16.1 MB | ✅ PASS |
| `redundant` | Redundant | 2.00 | 99.50% (201.53x) | N/A | 93.46 MB/s | N/A | 210.35 MB/s | 25.6 MB | ✅ PASS |
| `entropy` | Entropy | 2.00 | -0.08% (1.00x) | N/A | 104.92 MB/s | N/A | 176.09 MB/s | 17.5 MB | ✅ PASS |

## Compression Level Comparison (Levels 1 to 5)

| Corpus | Level | Block Size | Threads | Space Saved (%) | Ratio Factor | Bits/Byte | Comp Speed (MB/s) | Decomp Speed (MB/s) | Peak RAM (MB) | Status |
|---|---|---|---|---|---|---|---|---|---|---|
| `text` | Level 1 | 16 MB | 1 | 92.30% | 12.98x | 0.62 | 75.55 | 267.64 | 14.2 | PASS |
| `text` | Level 1 | 16 MB | 4 | 92.30% | 12.98x | 0.62 | 71.63 | 264.62 | 14.3 | PASS |
| `text` | Level 2 | 16 MB | 1 | 93.01% | 14.30x | 0.56 | 24.61 | 273.60 | 14.3 | PASS |
| `text` | Level 2 | 16 MB | 4 | 93.01% | 14.30x | 0.56 | 24.39 | 230.41 | 14.0 | PASS |
| `binary` | Level 1 | 16 MB | 1 | 82.17% | 5.61x | 1.43 | 52.63 | 166.01 | 15.8 | PASS |
| `binary` | Level 1 | 16 MB | 4 | 82.17% | 5.61x | 1.43 | 46.49 | 170.26 | 16.1 | PASS |
| `binary` | Level 2 | 16 MB | 1 | 84.08% | 6.28x | 1.27 | 16.29 | 181.23 | 14.1 | PASS |
| `binary` | Level 2 | 16 MB | 4 | 84.08% | 6.28x | 1.27 | 16.64 | 180.02 | 14.3 | PASS |
| `redundant` | Level 1 | 16 MB | 1 | 99.50% | 201.53x | 0.04 | 93.46 | 210.35 | 25.4 | PASS |
| `redundant` | Level 1 | 16 MB | 4 | 99.50% | 201.53x | 0.04 | 93.68 | 206.89 | 25.6 | PASS |
| `redundant` | Level 2 | 16 MB | 1 | 99.74% | 382.06x | 0.02 | 39.30 | 208.88 | 19.6 | PASS |
| `redundant` | Level 2 | 16 MB | 4 | 99.74% | 382.06x | 0.02 | 39.35 | 211.20 | 19.7 | PASS |
| `entropy` | Level 1 | 16 MB | 1 | -0.08% | 1.00x | 8.01 | 104.92 | 176.09 | 17.4 | PASS |
| `entropy` | Level 1 | 16 MB | 4 | -0.08% | 1.00x | 8.01 | 106.16 | 179.25 | 17.5 | PASS |
| `entropy` | Level 2 | 16 MB | 1 | -0.08% | 1.00x | 8.01 | 104.12 | 179.63 | 17.5 | PASS |
| `entropy` | Level 2 | 16 MB | 4 | -0.08% | 1.00x | 8.01 | 101.83 | 178.28 | 17.5 | PASS |

## Multi-Thread Scaling Analysis

Evaluates performance speedup and scaling efficiency across thread counts (1, 2, 4, 8 threads).

### Thread Scaling: Corpus `binary`

| Level | Threads | Comp Time (s) | Comp Speed (MB/s) | Comp Speedup | Decomp Time (s) | Decomp Speed (MB/s) | Decomp Speedup | Efficiency |
|---|---|---|---|---|---|---|---|---|
| Level 1 | 1 | 0.038s | 52.63 MB/s | 1.00x | 0.012s | 166.01 MB/s | 1.00x | 100.0% |
| Level 1 | 4 | 0.043s | 46.49 MB/s | 0.88x | 0.012s | 170.26 MB/s | 1.03x | 22.1% |
| Level 2 | 1 | 0.123s | 16.29 MB/s | 1.00x | 0.011s | 181.23 MB/s | 1.00x | 100.0% |
| Level 2 | 4 | 0.120s | 16.64 MB/s | 1.02x | 0.011s | 180.02 MB/s | 0.99x | 25.5% |

### Thread Scaling: Corpus `entropy`

| Level | Threads | Comp Time (s) | Comp Speed (MB/s) | Comp Speedup | Decomp Time (s) | Decomp Speed (MB/s) | Decomp Speedup | Efficiency |
|---|---|---|---|---|---|---|---|---|
| Level 1 | 1 | 0.019s | 104.92 MB/s | 1.00x | 0.011s | 176.09 MB/s | 1.00x | 100.0% |
| Level 1 | 4 | 0.019s | 106.16 MB/s | 1.01x | 0.011s | 179.25 MB/s | 1.02x | 25.3% |
| Level 2 | 1 | 0.019s | 104.12 MB/s | 1.00x | 0.011s | 179.63 MB/s | 1.00x | 100.0% |
| Level 2 | 4 | 0.020s | 101.83 MB/s | 0.98x | 0.011s | 178.28 MB/s | 0.99x | 24.5% |

### Thread Scaling: Corpus `redundant`

| Level | Threads | Comp Time (s) | Comp Speed (MB/s) | Comp Speedup | Decomp Time (s) | Decomp Speed (MB/s) | Decomp Speedup | Efficiency |
|---|---|---|---|---|---|---|---|---|
| Level 1 | 1 | 0.021s | 93.46 MB/s | 1.00x | 0.010s | 210.35 MB/s | 1.00x | 100.0% |
| Level 1 | 4 | 0.021s | 93.68 MB/s | 1.00x | 0.010s | 206.89 MB/s | 0.98x | 25.1% |
| Level 2 | 1 | 0.051s | 39.30 MB/s | 1.00x | 0.010s | 208.88 MB/s | 1.00x | 100.0% |
| Level 2 | 4 | 0.051s | 39.35 MB/s | 1.00x | 0.009s | 211.20 MB/s | 1.01x | 25.0% |

### Thread Scaling: Corpus `text`

| Level | Threads | Comp Time (s) | Comp Speed (MB/s) | Comp Speedup | Decomp Time (s) | Decomp Speed (MB/s) | Decomp Speedup | Efficiency |
|---|---|---|---|---|---|---|---|---|
| Level 1 | 1 | 0.026s | 75.55 MB/s | 1.00x | 0.007s | 267.64 MB/s | 1.00x | 100.0% |
| Level 1 | 4 | 0.028s | 71.63 MB/s | 0.95x | 0.008s | 264.62 MB/s | 0.99x | 23.7% |
| Level 2 | 1 | 0.081s | 24.61 MB/s | 1.00x | 0.007s | 273.60 MB/s | 1.00x | 100.0% |
| Level 2 | 4 | 0.082s | 24.39 MB/s | 0.99x | 0.009s | 230.41 MB/s | 0.84x | 24.8% |

## Integrity & Decompression Verification

All compressed archives were extracted and validated via MD5 checksum verification and byte-by-byte binary comparison (`cmp`).

| Corpus | Level | Threads | MD5 Input Checksum | MD5 Decompressed Checksum | Result |
|---|---|---|---|---|---|
| `text` | Level 1 | 1 | `dbaef03b58cfb10662bf81186260bbcc` | `dbaef03b58cfb10662bf81186260bbcc` | PASS |
| `text` | Level 1 | 4 | `dbaef03b58cfb10662bf81186260bbcc` | `dbaef03b58cfb10662bf81186260bbcc` | PASS |
| `text` | Level 2 | 1 | `dbaef03b58cfb10662bf81186260bbcc` | `dbaef03b58cfb10662bf81186260bbcc` | PASS |
| `text` | Level 2 | 4 | `dbaef03b58cfb10662bf81186260bbcc` | `dbaef03b58cfb10662bf81186260bbcc` | PASS |
| `binary` | Level 1 | 1 | `36befa42509970aedd283330efe814be` | `36befa42509970aedd283330efe814be` | PASS |
| `binary` | Level 1 | 4 | `36befa42509970aedd283330efe814be` | `36befa42509970aedd283330efe814be` | PASS |
| `binary` | Level 2 | 1 | `36befa42509970aedd283330efe814be` | `36befa42509970aedd283330efe814be` | PASS |
| `binary` | Level 2 | 4 | `36befa42509970aedd283330efe814be` | `36befa42509970aedd283330efe814be` | PASS |
| `redundant` | Level 1 | 1 | `afb6783b63a5d063a1db4d6e29a4421d` | `afb6783b63a5d063a1db4d6e29a4421d` | PASS |
| `redundant` | Level 1 | 4 | `afb6783b63a5d063a1db4d6e29a4421d` | `afb6783b63a5d063a1db4d6e29a4421d` | PASS |
| `redundant` | Level 2 | 1 | `afb6783b63a5d063a1db4d6e29a4421d` | `afb6783b63a5d063a1db4d6e29a4421d` | PASS |
| `redundant` | Level 2 | 4 | `afb6783b63a5d063a1db4d6e29a4421d` | `afb6783b63a5d063a1db4d6e29a4421d` | PASS |
| `entropy` | Level 1 | 1 | `d4120e4b2e5c20393e39cc86d23bb81b` | `d4120e4b2e5c20393e39cc86d23bb81b` | PASS |
| `entropy` | Level 1 | 4 | `d4120e4b2e5c20393e39cc86d23bb81b` | `d4120e4b2e5c20393e39cc86d23bb81b` | PASS |
| `entropy` | Level 2 | 1 | `d4120e4b2e5c20393e39cc86d23bb81b` | `d4120e4b2e5c20393e39cc86d23bb81b` | PASS |
| `entropy` | Level 2 | 4 | `d4120e4b2e5c20393e39cc86d23bb81b` | `d4120e4b2e5c20393e39cc86d23bb81b` | PASS |
