# NOW -- an open digest for bitwalk --fasm, and the L3 row measured alike for every tool (2026-10-04)

## specs/xilinx7/fasm_digest.py, flow.t27, l3_bench.py (Closes #6068)

- `fasm_digest.py DB_ROOT PART` writes the prjxray-db digest that `bitwalk --fasm` reads, with the
  Python standard library only. Until now a local helper (`x7.py fasm_digest`) wrote it, so
  nobody else could run `bitwalk --fasm` (gHashTag/trinity#1272, third box).
  - One trap, kept in the code: a K line's shift is in BITS, 32 x the alias `start_offset` (frames.t27
    `seg_pos` / `seg_in_window` take bits). Written in words, 8 of 20,230 frames of an AX7203
    design came out wrong, all in `*_SING` IO tiles.
- `flow.t27` is the arithmetic of the #/devkit table in integer milliseconds: best and worst of N,
  replaced layer, saved time, speedup and share in thousandths (half up), the Amdahl ceiling of
  one more layer, tenths of an hour a year, and a verdict: faster only if the rewrite's slowest
  run beats the old tool's fastest, slower only the other way round, otherwise a tie.
  8/8 tests; mutation check 9 of 10 killed, the tenth (`after <= before` in `saved`) is
  equivalent. Vectors: `conformance/xilinx7_flow.json`, replayed by `bitwalk --flow`
  (16/16; a vector with a wrong expectation turns it red).
- `l3_bench.py` + `.github/workflows/xilinx7-l3-bench.yml`: one runner, every tool N times in a
  rotated order, `.frames` and `.bit` byte-compared before anything is timed.

## Measured

- On this Mac, prjxray-db a90f27c1, xc7a200tfbg484-2: `bitwalk --fasm` with the open digest
  is byte-identical to the `.frames` the openXC7 image's fasm2frames wrote for 42 of 42
  AX7203 builds of dmitrii-f-t27/trinity-memory (113,257-304,079 FASM lines). The
  2026-10-03 corpus had no xc7a200t file.

## What this does not show

- No timing yet: the bench numbers come from the workflow run, recorded in the PR.
- The corpus of the workflow is four of those designs (release corpus-xc7a200t-fasm-1 of
  dmitrii-f-t27/trinity-memory); the DDR3 designs are left out because they contain GPL RTL.
- Required features and STEPDOWN are not exercised by any artix7 corpus file.
