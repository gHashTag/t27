# FPGA Loop Closeout Report — Wave Loop 660

**Issue:** [#1631](https://github.com/ai-grid/t27/issues/1631)  
**Branch:** `wave-loop-660`  
**Date:** 2026-07-07  
**Variant:** A — module-scope `[139][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable initialized from a function call, with indexed signed field writes and read-back.

---

## 1. What was built

A witness `specs/scratch/w660_bench_module_139x2p6_aos_var_call_write.t27` that declares a module-scope mutable packed `reg` of type `[139][2][2][2][2][2][2]Pt` (where `Pt { x: i16, y: i16 }`), initializes it from a function call that returns the same shape, then performs indexed signed field writes and read-back checks.

- **Outer dimension:** 139 (non-power-of-two).
- **Total elements:** 139 × 2⁶ = 8,896.
- **Packed vector width:** 8,896 × 32 = 284,672 bits (≈0.271 MiBit).
- **Schedule:** signed i16 values `(2*e + offset) % 32768` with explicit `make_grid(32768)` modulo-wrap check, plus fixed sanity constants.
- **Tail checks:** first element, last element, mid-row element, and sanity constants.

---

## 2. Verification results

| Gate | Command / Test | Result |
|------|----------------|--------|
| Parse | `t27c parse specs/scratch/w660_bench_module_139x2p6_aos_var_call_write.t27` | PASS |
| Icarus lowerable | `t27c icarus-lowerable --json ...` | `{ "lowerable": true }` |
| Icarus simulate | `t27c icarus-simulate ...` | PASSED (17 cycles) |
| Cocotb cross-check | `t27c icarus-cocotb ...` | reference-model OK |
| Seal | `t27c seal --save ...` | saved, hashes match |
| t27c binary tests | `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| tri tests | `cargo test -p tri` | 78 passed; 0 failed |
| Icarus lowerable tests | `cargo test -p t27c --test icarus_lowerable` | 120 passed; 0 failed |
| Local sweep | `./scripts/tri test --fast` | parse/typecheck/gen-Zig/gen-Rust/gen-Verilog/gen-C/seal/fixed-point clean |

The `./scripts/tri test --fast` sweep reports **24 pre-existing Gen Verilog Yosys Smoke failures** caused by `translate_off` comment warnings. These failures are unrelated to the packed-AoS ladder and were already present in prior waves; they are not treated as blockers.

**FROZEN_HASH** remains unchanged at `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

---

## 3. Changes made

- `scripts/gen_w660.py` — generator for the W660 witness (copied from `gen_w659.py`, `OUTER = 139`).
- `specs/scratch/w660_bench_module_139x2p6_aos_var_call_write.t27` — generated witness (~607 KB, 26,471 lines).
- `bootstrap/tests/icarus_lowerable.rs` — added integration test `accepts_w660_bench_module_139x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w660_bench_module_139x2p6_aos_var_call_write.json` — saved seal.
- `.trinity/icarus-baselines/specs/scratch/w660_bench_module_139x2p6_aos_var_call_write.json` — empty Icarus baseline `{"lines":[]}`.
- `.trinity/current-issue.md` — updated for W660; carries forward W661 cooperation variants.
- `.claude/plans/wave-loop-660.md` — decomposed plan for W660.
- `docs/reports/FPGA_LOOP_CLOSEOUT_W660_2026-07-07.md` — this report.
- `.trinity/experience.md` — updated with W660 learnings.

---

## 4. Weak points addressed

1. **First outer dimension 139.** Confirmed the compiler and reference model correctly stride by 139 for a non-power-of-two outer dimension.
2. **Modulo-wrap signal.** The explicit `make_grid(32768)` check preserves the same wrap regression coverage as earlier waves even though the element count (8,896) is below the natural wrap point.
3. **Multi-line literal brace style.** Reused the W584/W659 multi-line brace style so the 6-D nested literal parses correctly.
4. **Simulator capacity.** At ≈0.271 MiBit the witness simulates in 17 cycles, well below the Icarus very-large-vector threshold.
5. **`assert_ne` Icarus emission gap.** Continued using `assert_eq` checks on changed elements, since the structural classifier accepts `assert_ne` but `gen_verilog_test_stmt` does not emit it.

---

## 5. Scientific / technical background

- **IEEE Std 1800-2017** §7.4.1/7.4.3 — packed-array total width is the product of packed dimensions; ranges need not be powers of two.
- **Sutherland, “Synthesizable SystemVerilog”** — packed arrays and packed structs as synthesizable first-class objects.
- **Icarus Verilog Quirks / Extensions pages** — width handling and packed-array subset behavior.
- **Icarus issue #1134** — assertion failures with unpacked arrays of packed structs; t27 flattening avoids the trigger.
- **Icarus issue #1171** — freezes during elaboration of very large packed vectors; W660 stays far below the reported threshold.
- **Yosys docs / PR #4100 / issue #4653 / issue #2677** — multidimensional packed arrays supported, arrays of packed structs unsupported; t27 flattening avoids the gap.
- **cocotb PR #3608 / discussion #2933** — packed structs as whole signals; flat `LogicArray` for multidimensional packed arrays in the reference model.
- **Lutsig (CPP 2021)** — verified array-read lowering.
- **CIRCT `HWLegalizeModules.cpp` / SV dialect** — production packed-array scalarization.

---

## 6. L1–L7 compliance

| Law | Status | Evidence |
|-----|--------|----------|
| L1 TRACEABILITY | ✅ | This report and commits reference `Closes #1631`. |
| L2 GENERATION | ✅ | `gen/` untouched; all generated artifacts produced by `t27c`. |
| L3 PURITY | ✅ | Source identifiers ASCII/English; witness uses approved style. |
| L4 TESTABILITY | ✅ | Witness contains `bench` block with assertions and cocotb cross-check. |
| L5 IDENTITY | ✅ | No math/physics changes; φ invariants unaffected. |
| L6 CEILING | ✅ | No numeric SSOT files touched. |
| L7 UNITY | ✅ | No new `*.sh` on critical path; used `tri`/`t27c`. |

---

## 7. Next Wave Loop 661 cooperation variants

1. **Variant A — `[141][2]^6 Pt` module-scope var from a call with indexed signed writes.**  
   288,768-bit packed vector, 9,024 elements, non-power-of-two outer dimension 141, well under the 4-MiBit cliff. Continues the odd outer-dimension ladder. **Recommended.**

2. **Variant B — `[139][2]^6 Pt` bench-local (function-local) packed array var from a call with indexed signed writes.**  
   284,672-bit packed vector, 8,896 elements. Tests that the same non-p2 outer dimension works when the mutable `reg` is declared inside a bench/function rather than at module scope. Useful complement to the module-scope ladder.

3. **Variant C — `[139][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes.**  
   Stays at 0.271 MiBit and tests that control-flow guarded indexed writes on a packed `reg` are correctly elaborated and simulated (e.g. write only when a signed index exceeds a threshold). Useful follow-up to W590/W591.

---

## 8. Commit summary

- Feature commit: `feat(igla): Wave Loop 660 — module-scope [139][2]^6 Pt non-p2 AoS var from a call with indexed signed writes`  
  `Closes #1631`
- Tracking commit: records session log, commit count, plan, closeout report, seal, baseline, and experience update.

---

Phase complete: Verify
→ Phase 9: Learn
