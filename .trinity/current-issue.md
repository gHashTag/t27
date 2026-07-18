# Wave Loop 585 — Current Issue

**Issue #1556** — Next step after 17-D array-of-struct return call
sededuplication (W584).
**Branch:** `wave-loop-585` (to be created from `wave-loop-584`).
**Previous:** Wave Loop 584 closed (#1555, branch `wave-loop-584`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant C is recommended because the W584
17-D direct simulation already took ~22.5 minutes, making further rank scaling
risky for CI. Variants A and B remain documented options if the user wants to
continue pushing the width boundary.

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

3. **Variant C — Recommended: module-scope 7-D array-of-struct variable
   initialized from a call, with multi-site bench CSE.**
   Add a witness with a module-level `var dst : [2][2][2][2][2][2][2]Pt`
   (524,288-bit packed width, 16,384 elements) initialized from a function
   call returning a 7-D AoS with computed fields. The bench/test blocks should
   read `dst` at multiple whole-array and indexed sites, exercising W557
   call-array CSE across module-scope and function-return boundaries. Keeps
   file size small while covering scope/CSE interaction.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w585_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W586
  variants recorded in `.trinity/current-issue.md`.
