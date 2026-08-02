# FPGA Loop Closeout — Wave Loop 836 (2026-08-01)

**Wave:** 836  
**Issue:** #1612  
**Branch:** `wave-loop-836`  
**Parent branch:** `wave-loop-835` HEAD because earlier wave PRs remain open  
**PR:** #1613  
**Variant:** A — module-scope `[491][2]^6 Pt` packed array-of-struct variable from call with indexed signed writes

---

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[491][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or `FROZEN_HASH` changes.

## What changed

- Added generator `scripts/gen_w836.py` (copied from `scripts/gen_w835.py` with the
  recurring copy hazard fixed: destination path, module header f-string, and
  `MID_IDX` comment updated to `w836` / `491` / `245`).
- Generated `specs/scratch/w836_bench_module_491x2p6_aos_var_call_write.t27`
  (31,424 elements, 1,005,568-bit packed vector, ~0.959 MiBit).
- Added integration test `accepts_w836_bench_module_491x2p6_aos_var_call_write` to
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
| `cargo test -p t27c --test icarus_lowerable --release` | 296 passed; 0 failed |
| `t27c parse` W836 | PASS |
| `t27c icarus-lowerable` W836 | `lowerable` |
| `t27c icarus-simulate` W836 | `PASSED` (17 cycles) |
| `t27c icarus-cocotb` W836 | `reference-model OK` |
| `t27c seal --save` W836 | seal saved |
| `FROZEN_HASH` | unchanged (`68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`) |

## Scientific / engineering background

- IEEE 1800-2017 §7.4.1/7.4.3 define packed-array width as the product of packed
  dimensions, with no power-of-two restriction. Variant A emits a single
  1,005,568-bit packed vector, which is legal SystemVerilog.
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
- Generator copy hazard persists; parameterize `WAVE`/`OUTER` in the template.

## Cooperation variants for Wave Loop 837

- **A (recommended):** `[493][2]^6 Pt`, outer += 2, `MID_IDX = 246`.
  Smallest mechanical increment; expected 31,552 elements, 1,009,664 bits (~0.963 MiBit).
- **B:** `[491][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
  Expected 357,441 elements, ~11.43 MiBit; may cross the 4-MiBit cliff or expose a
  backend width/stride bug. Convert to negative boundary witness if blocked.
- **C:** `[491][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

*φ² + φ⁻² = 3 | TRINITY*
