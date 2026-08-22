# NOW -- "the 32-module fpga set" was a literal, and it had drifted both ways (2026-08-23)

Refs #2325.

- `fpga-build` iterated a hardcoded 36-name array in `bootstrap/src/main.rs`,
  not `specs/fpga/`. Measured drift in BOTH directions:
  - **5 names have no spec** -- `sv_emit`, `firrtl`, `cdc`, `lint`, `coverage`
    -- and printed `SKIP (spec not found)` while the command exited 0.
  - **4 specs were never generated at all**: `bpsk` (the merged BPSK modem
    core), `power_analysis`, `ternary_link`, `vcd_conformance_compare`.
- Generating those four with this same `t27c` and the ratchet's own iverilog
  invocation: **14 errors nothing counted and nothing bounded** --
  `vcd_conformance_compare` 13, `power_analysis` 1, the other two clean.
- The set is now the directory. `SKIP` disappears because there is nothing to
  skip, and an empty directory fails loudly rather than reporting success over
  zero modules.
- The elaboration ratchet did its job against me: it refused the change with
  four `NEW` rows and exit 1 until the increase was recorded deliberately.
  Baseline 162/32 -> **176/36**, with the reason written next to the number --
  this is coverage arriving, not a regression. The earlier hand-written note
  survived the update, which is the preservation added in #2441 working.
- Everything else held: yosys 36/36, conformance still executes 18 mac + 3 spi,
  vector and withdrawn gates green. The 13 red cargo tests are unchanged and
  pre-existing (#2292).
