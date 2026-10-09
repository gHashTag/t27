# NOW -- sieve C6 and C7: t27b warmup measured, speed and quality published (2026-10-09)

## three cells FALSE to TRUE, each with the spec that measured it (Closes #7596)

- `specs/compiler/theory/warmup.t27` (T750) is the Barrett et al. 2017 classifier in t27: outliers,
  optimal partitioning on mean and variance, the steady-state rule, the per-benchmark class. 8 tests on
  synthetic series, 0 vacuous; a mutant that reads WARMUP as FLAT fails 3 of them.
- `warmup_t27b.t27` (T751, tool-written) holds the 30000 iteration times of the lab run: six kernels,
  500 in-process iterations x 10 executions, t27b under qemu-aarch64. sieve, collatz, ternary FLAT;
  fib, matmul GOOD INCONSISTENT; f64loop BAD INCONSISTENT. No first-iteration spike.
- `speed_quality.t27` (T753) holds 5 samples per row for t27b and for t27c through gcc, clang and zig;
  its tests recompute the medians and geomeans. t27b is 3.13x clang -O2 (wall, 5 kernels) and 2.27x
  zig ReleaseFast (6 kernels); gen-c takes 3.5-5.7 ms, clang -O2 compiles its C in 22x that.
- sieve.t27 T752: t27b C6, t27b C7, t27c C7 FALSE to TRUE; t27c now 3 FALSE, t27b 1; G6 PARTIAL.
  Limits: emulated arm64, six kernels, one run. Report: docs/reports/2026-10-09-sieve-c6-c7.md.
