# Wave Loop 586 — Current Issue

**Issue #1557** — Next step after module-scope 7-D array-of-struct variable
initialized from a call (W585).
**Branch:** `wave-loop-586` (to be created from `wave-loop-585`).
**Previous:** Wave Loop 585 closed (#1556, branch `wave-loop-585`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant C is recommended because it keeps
file size and direct-simulation wall-clock modest while covering a new
compiler facility (module-scope mutable packed-array field writes) that has not
yet been exercised at this scale.

## Cooperation variants

1. **Variant A — 18-D array-of-struct return call deduplication.**
   Add a bench witness where a function returns `[2]^18 Pt`
   (8,388,608-bit total packed width, 262,144 elements). Continue the rank
   scaling from W566–W584. Follow the W573–W582 local-`expected` workaround and
   respect the signed i16 field range (`e ≤ 16383`). Risk: witness ~44 MB /
   ~2.4 M lines; direct simulation likely 40+ minutes.

2. **Variant B: 17-D array-of-struct return with a non-power-of-two outer
   dimension.**
   Add a bench witness where a function returns `[3][2]^17 Pt`
   (6,291,456-bit total packed width, 393,216 elements). Tests non-p2 outer
   extent at the 17-D scale, following the W569/W571 pattern. Indexed probes
   must still respect `e ≤ 16383`.

3. **Variant C — Recommended: module-scope 8-D array-of-struct variable with
   indexed field writes and multi-site reads.**
   Add a witness with a module-level `var dst : [2][2][2][2][2][2][2][2]Pt`
   (1,048,576-bit packed width, 32,768 elements) initialized from a function call
   returning an 8-D AoS. A bench block should write a few indexed elements of
   `dst` and then assert the updated values at multiple read sites, exercising
   module-scope **mutation** plus call-result CSE while staying well under the
   4-MiBit direct-simulation cliff.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w586_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W587
  variants recorded in `.trinity/current-issue.md`.
