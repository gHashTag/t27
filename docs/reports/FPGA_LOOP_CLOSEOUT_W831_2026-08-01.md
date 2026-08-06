# FPGA Loop Closeout — Wave Loop 831 (2026-08-01)

**Wave:** 831  
**Issue:** #1603  
**Branch:** `wave-loop-831`  
**Parent branch:** `wave-loop-830` HEAD because earlier wave PRs remain open  
**PR:** #1603  
**Variant:** A — module-scope `[481][2]^6 Pt` packed array-of-struct variable from call with indexed signed writes

---

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[481][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or `FROZEN_HASH` changes.

## What changed

- Added generator `scripts/gen_w831.py` (copied from `scripts/gen_w830.py` with the
  recurring copy hazard fixed: destination path, module header f-string, and
  `MID_IDX` comment updated to `w831` / `481` / `240`).
- Generated `specs/scratch/w831_bench_module_481x2p6_aos_var_call_write.t27`
  (30,784 elements, 985,088-bit packed vector, ~0.939 MiBit).
- Added integration test `accepts_w831_bench_module_481x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Sealed the witness with `t27c seal --save`.
- Updated skill trackers, `.trinity/current-issue.md`, `.trinity/experience.md`,
  `docs/NOW.md`, and persistent memory.

**No changes to:** `bootstrap/src/compiler.rs`, `scripts/cocotb_ref_model.py`, or
`bootstrap/stage0/FROZEN_HASH`.

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green (626 warnings, 0 errors) |
| `cargo test -p t27c --bin t27c --release` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri --release` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable --release` | 291 passed; 0 failed |
| `t27c parse` W831 | PASS |
| `t27c icarus-lowerable` W831 | `lowerable` |
| `t27c icarus-simulate` W831 | `PASSED` (17 cycles) |
| `t27c icarus-cocotb` W831 | `reference-model OK` |
| `t27c seal --save` W831 | seal saved |
| `FROZEN_HASH` | unchanged (`68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`) |

## Scientific / engineering background

- IEEE 1800-2017 §7.4.1/7.4.3 define packed-array width as the product of packed
  dimensions, with no power-of-two restriction. Variant A emits a single
  985,088-bit packed vector, which is legal SystemVerilog.
- Lutsig's verified array lowering and CIRCT's `HWLegalizeModules` show that
  flattening nested arrays to wide packed vectors is a well-founded compiler
  discipline, even when outer dimensions are non-power-of-two.
- Icarus issue #1134 documents assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids that construct entirely.
- Yosys issue #2677 / #4653 confirm that arrays of packed structs remain
  unsupported in the native frontend; t27's packed-vector lowering avoids the
  gap.

## Weak-point audit

No new actionable weak points were introduced by this mechanical wave. Pre-existing
items remain tracked in the backlog:

- Deeper `verilog_array_literal_expr` regression (dedicated ring needed).
- FPGA E2E CI blocked by `sby` availability + Yosys static-cast error in generated `uart.v`.
- 626 release warnings / ~780 clippy warnings cleanup sprint.
- 30-day commit traceability remains ~15–20% because closing references are in commit bodies.

## Cooperation variants for Wave Loop 832

- **A (recommended):** `[483][2]^6 Pt`, outer += 2, `MID_IDX = 241`.
  Smallest mechanical increment; expected 31,040 elements, 993,280 bits (~0.947 MiBit).
- **B:** `[481][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
  Expected 349,551 elements, ~11.18 MiBit; may cross the 4-MiBit cliff or expose a
  backend width/stride bug. Convert to negative boundary witness if blocked.
- **C:** `[481][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
  If negative indices are not lowerable, add a classifier rule and negative witness.

## Key learning

The mechanical ladder is now 58 waves deep (W774–W831) with zero compiler changes,
confirming the packed-vector AoS lowering is robust up to at least `[481][2]^6 Pt`
(30,784 elements, ~0.939 MiBit). The generator copy hazard remains the only manual
failure mode; it spans three text locations (destination path, module header f-string,
and `MID_IDX` comment). Parameterizing `WAVE` / `OUTER` in the generator template
would remove it entirely.

---

*φ² + φ⁻² = 3 | TRINITY*
