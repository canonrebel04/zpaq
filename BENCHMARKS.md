# Benchmarks and format compatibility

## Format compatibility

ZPAQ archives carry their own model, and the decoder must reproduce the
encoder's predictions exactly. Any change to how a component predicts or
learns therefore changes the archive format.

Up to dead3bd the fork had changed two things in the interpreted (`NOJIT`)
predictor:

- `Predictor::find`, the ICM/ISSE/CM hash-table bucket hash (clmul or
  `cxt*0xbf58476d1ce4e5b9>>32` instead of the spec's `cxt*16`);
- the MIX/MIX2 weight update (a momentum term instead of the spec update).

The JIT predictor (`assemble_p`) still inlined the original code. So
`-m3`, `-m4`, `-m5` and custom CM models written by the fork could not
be read by zpaq 7.15, and the reverse was also true. Even the fork's own JIT and
NOJIT builds could not read each other's archives. `-m1` and `-m2` were
not affected because they use no context-model components.

This was a bug rather than an intentional format change. Nothing marked the
change (no new level or header), the JIT path was not changed with it, and
the momentum update also compressed slightly worse (`-m4` 3,621,311 B
against 3,608,041 B). Both are now back to the specified behaviour, and the
mixer update is vectorised with AVX2 in a way that is bit-exact.

The fork, zpaq 7.15 JIT and zpaq 7.15 NOJIT now read each other's archives
at every level and produce byte-identical archive sizes:

- `tests/fixtures.sh` passes 14/14 against archives made by 7.15. The old
  fork passes 6/14.
- `tests/roundtrip.sh` passes 36/36 against 7.15.

Archives written by the old fork at `-m3`..`-m5` or with custom CM models
can only be extracted with that old build.

## Speed

Measured with `tests/bench.sh` on `text-all.bin` (30,185,772 bytes of
crawled web text) using `-t1`. The machine was an Intel(R) Xeon(R) Processor (8 vCPUs, SHA-NI and
AVX2), with g++ (Debian 14.2.0-19) 14.2.0. Every extraction was checked against the input.

| Build | Notes |
|---|---|
| before | the fork at dead3bd (Makefile defaults) |
| up-jit / up-nojit | unmodified zpaq 7.15, `-O3 -march=native` |
| new-jit | this branch, `make` |
| new-nojit | this branch with `-DNOJIT` |

| method | build | archive bytes | ratio | compress MB/s | extract MB/s | ok |
|---|---|---:|---:|---:|---:|---|
| -m1 | before | 5426563 | 5.563 | 41.79 | 18.89 | yes |
| -m1 | up-jit | 5426563 | 5.563 | 36.65 | 117.57 | yes |
| -m1 | up-nojit | 5426563 | 5.563 | 36.35 | 20.35 | yes |
| -m1 | new-jit | 5426563 | 5.563 | 40.34 | 218.31 | yes |
| -m1 | new-nojit | 5426563 | 5.563 | 39.39 | 18.60 | yes |
| -m2 | before | 4953529 | 6.094 | 7.70 | 18.76 | yes |
| -m2 | up-jit | 4953529 | 6.094 | 8.46 | 136.33 | yes |
| -m2 | up-nojit | 4953529 | 6.094 | 7.44 | 19.93 | yes |
| -m2 | new-jit | 4953529 | 6.094 | 7.79 | 219.01 | yes |
| -m2 | new-nojit | 4953529 | 6.094 | 7.55 | 18.29 | yes |
| -m3 | before | 4444543 | 6.792 | 4.87 | 7.77 | yes |
| -m3 | up-jit | 4429433 | 6.815 | 5.96 | 15.10 | yes |
| -m3 | up-nojit | 4429433 | 6.815 | 5.21 | 8.23 | yes |
| -m3 | new-jit | 4429433 | 6.815 | 6.04 | 16.04 | yes |
| -m3 | new-nojit | 4429433 | 6.815 | 5.04 | 7.73 | yes |
| -m4 | before | 3621311 | 8.336 | 1.18 | 1.12 | yes |
| -m4 | up-jit | 3608041 | 8.366 | 1.71 | 1.67 | yes |
| -m4 | up-nojit | 3608041 | 8.366 | 1.26 | 1.22 | yes |
| -m4 | new-jit | 3608041 | 8.366 | 1.71 | 1.69 | yes |
| -m4 | new-nojit | 3608041 | 8.366 | 1.27 | 1.17 | yes |
| -m5 | before | 3416372 | 8.836 | 0.35 | 0.35 | yes |
| -m5 | up-jit | 3408010 | 8.857 | 0.55 | 0.55 | yes |
| -m5 | up-nojit | 3408010 | 8.857 | 0.37 | 0.35 | yes |
| -m5 | new-jit | 3408010 | 8.857 | 0.56 | 0.54 | yes |
| -m5 | new-nojit | 3408010 | 8.857 | 0.37 | 0.37 | yes |

Where the speed comes from:

- `-m1`/`-m2` decoding is mostly PCOMP (LZ77 post-processing) plus SHA-1. The
  JIT post-processor used to return to C++ once per output byte; it now runs
  over 4 KB batches, with `out` inlined. SHA-1 uses the SHA-NI instructions
  when the CPU has them. Together these take extraction from 118-136 MB/s
  (7.15 JIT) to about 218 MB/s.
- At `-m3`..`-m5` the time goes into the context models. These run in
  JIT-compiled code that is the same as 7.15's, because the format requires
  identical predictions. Speed is therefore within noise of 7.15 JIT, about
  6% faster at `-m3` from the faster decode path.
- Ratio is now identical to 7.15 at every level.
