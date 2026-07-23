# FPGA Wave Loop 631 Closeout Report

**Issue:** #1602  
**Branch:** `wave-loop-631`  
**Date:** 2026-07-07  
**Variant:** A — `[81][2]^6 Pt` module-scope non-power-of-two AoS variable
initialized from a function call, with indexed signed field writes and read-back.

---

## 1. What was delivered

- `specs/scratch/w631_bench_module_81x2p6_aos_var_call_write.t27`
  - Module-scope `[81][2][2][2][2][2][2] Pt` (5,184 elements, 165,888-bit packed
    vector, ≈0.158 MiBit).
  - `pub fn make_grid(offset : u16) -> [81][2][2][2][2][2][2] Pt` returning a
    fully expanded multi-line literal.
  - `pub const expected` and `pub var dst` both initialized from `make_grid(0)`.
  - `test module_var_81x2p6_call_write` and `bench module_bench_81x2p6_call_write`
    exercising whole-array equality, corner indexed reads, signed writes,
    read-back, frame-condition checks, and changed-element checks.
- Integration test `accepts_w631_bench_module_81x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Icarus baseline `.trinity/icarus-baselines/specs/scratch/w631_bench_module_81x2p6_aos_var_call_write.json`.
- Seal `.trinity/seals/scratch_w631_bench_module_81x2p6_aos_var_call_write.json`.
- Generator script `scripts/gen_w631.py`.

---

## 2. Weak points investigated

| # | Weak point | Finding |
|---|------------|---------|
| 1 | **Outer dimension 81** | Stride-by-81 layout works end-to-end; no compiler changes required. |
| 2 | **Modulo-wrap signal** | 5,184 elements are below the wrap point; explicit `make_grid(32768)` preserves the regression signal. |
| 3 | **Mega-literal parsing** | Multi-line W584-style brace literal remains mandatory. A malformed dimension annotation (`[81][][][][][][]Pt`) was accepted by the parser but produced wrong layout; the final witness uses explicit `[81][2][2][2][2][2][2]Pt`. |
| 4 | **Simulator capacity** | 0.158 MiBit is comfortably interactive. |
| 5 | **`assert_ne` lowerability** | The structural classifier accepts `assert_ne`, but the Icarus simulation path emits it as a raw Verilog task and iverilog rejects it. W631 avoids the whole-array `assert_ne(dst, expected)` used in earlier waves and instead checks the changed elements with `assert_eq`. |

---

## 3. Verification matrix

| Gate | Command | Result |
|------|---------|--------|
| Parse | `t27c parse specs/scratch/w631_bench_module_81x2p6_aos_var_call_write.t27` | PASS |
| Icarus lowerable | `t27c icarus-lowerable ...` | `lowerable` |
| Icarus simulate | `t27c icarus-simulate ...` | PASS, silent exit 0 |
| Reference model | `t27c icarus-cocotb ...` | `reference-model OK` |
| Seal | `t27c seal --save ...` | saved |
| `cargo test -p t27c --bin t27c` | — | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | — | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | — | 91 passed; 0 failed |

FROZEN_HASH unchanged: `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

---

## 4. Scientific / technical background

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array width is the product of packed
  dimensions, with no power-of-two restriction.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are first-class
  synthesizable objects.
- Icarus issue #1134 — unpacked arrays of packed structs can assert; t27 flattening
  avoids the trigger.
- Icarus issue #1171 — large packed vectors can freeze elaboration; W631 stays far
  below the reported threshold.
- Yosys issues #2677, #4653, PR #4100 — multidimensional packed arrays supported,
  arrays of packed structs not; t27 flattening avoids the gap.
- cocotb `LogicArray` — Python reference model mirrors row-major LSB-first layout.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` — production packed-array scalarization aligns with
  t27's flattening discipline.

---

## 5. Next Wave Loop 632 cooperation variants

1. **Variant A — `[83][2]^6 Pt` module-scope var from a call with indexed signed
   writes.** 169,984-bit, 5,312 elements, continues the odd outer-dimension ladder
   under the 4-MiBit cliff. **Recommended.**
2. **Variant B — `[2]^19 Pt` module-scope var from a call.** 16.78 MiBit, crosses
   the 4-MiBit cliff by 4×; risky without chunked literals.
3. **Variant C — `[81][2]^6 Pt` with `if`-guarded whole-array reassignment then
   indexed signed writes.** Stays at 0.158 MiBit and tests control-flow guarded
   packed `reg` reassignment.

---

## 6. Files changed

- `specs/scratch/w631_bench_module_81x2p6_aos_var_call_write.t27` (new)
- `.trinity/seals/scratch_w631_bench_module_81x2p6_aos_var_call_write.json` (new)
- `.trinity/icarus-baselines/specs/scratch/w631_bench_module_81x2p6_aos_var_call_write.json` (new)
- `bootstrap/tests/icarus_lowerable.rs` (+1 integration test)
- `scripts/gen_w631.py` (new)
- `.trinity/current-issue.md` (updated for W632 variants)
- `.claude/plans/wave-loop-631.md` (new)
- `.trinity/experience.md` (W631 learnings appended)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W631_2026-07-07.md` (this report)

---

Phase complete: Wave Loop 631 closeout
