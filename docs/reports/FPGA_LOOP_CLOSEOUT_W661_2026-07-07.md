# FPGA Loop Closeout Report — Wave Loop 661

**Issue:** [#1632](https://github.com/ai-grid/t27/issues/1632)  
**Branch:** `wave-loop-661`  
**Date:** 2026-07-07  
**Variant:** A — module-scope `[141][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable initialized from a function call, with indexed signed field writes and read-back.

---

## 1. What was built

A witness `specs/scratch/w661_bench_module_141x2p6_aos_var_call_write.t27` that declares a module-scope mutable packed `reg` of type `[141][2][2][2][2][2][2]Pt` (where `Pt { x: i16, y: i16 }`), initializes it from a function call that returns the same shape, then performs indexed signed field writes and read-back checks.

- **Outer dimension:** 141 (non-power-of-two).
- **Total elements:** 141 × 2⁶ = 9,024.
- **Packed vector width:** 9,024 × 32 = 288,768 bits (≈0.275 MiBit).
- **Schedule:** signed i16 values `(2*e + offset) % 32768` with explicit `make_grid(32768)` modulo-wrap check, plus fixed sanity constants.
- **Tail checks:** first element, last element, mid-row element, and sanity constants.

---

## 2. Verification results

| Gate | Command / Test | Result |
|------|----------------|--------|
| Parse | `t27c parse specs/scratch/w661_bench_module_141x2p6_aos_var_call_write.t27` | PASS |
| Icarus lowerable | `t27c icarus-lowerable --json ...` | `{ "lowerable": true }` |
| Icarus simulate | `t27c icarus-simulate ...` | PASSED (17 cycles) |
| Cocotb cross-check | `t27c icarus-cocotb ...` | reference-model OK |
| Seal | `t27c seal --save ...` | saved, hashes match |
| t27c binary tests | `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| tri tests | `cargo test -p tri` | 78 passed; 0 failed |
| Icarus lowerable tests | `cargo test -p t27c --test icarus_lowerable` | 121 passed; 0 failed |
| Local sweep | `./scripts/tri test --fast` | parse/typecheck/gen-Zig/gen-Rust/gen-Verilog/gen-C/seal/fixed-point clean |

The `./scripts/tri test --fast` sweep reports **24 pre-existing Gen Verilog Yosys Smoke failures** caused by `translate_off` comment warnings. These failures are unrelated to the packed-AoS ladder and were already present in prior waves; they are not treated as blockers.

**FROZEN_HASH** remains unchanged at `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

---

## 3. Changes made

- `scripts/gen_w661.py` — generator for the W661 witness (copied from `gen_w660.py`, `OUTER = 141`).
- `specs/scratch/w661_bench_module_141x2p6_aos_var_call_write.t27` — generated witness (~616 KB, 26,851 lines).
- `bootstrap/tests/icarus_lowerable.rs` — added integration test `accepts_w661_bench_module_141x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w661_bench_module_141x2p6_aos_var_call_write.json` — saved seal.
- `.trinity/icarus-baselines/specs/scratch/w661_bench_module_141x2p6_aos_var_call_write.json` — empty Icarus baseline `{"lines":[]}`.
- `.trinity/current-issue.md` — updated for W661; carries forward W662 cooperation variants.
- `.claude/plans/wave-loop-661.md` — decomposed plan for W661.
- `docs/reports/FPGA_LOOP_CLOSEOUT_W661_2026-07-07.md` — this report.
- `.trinity/experience.md` — updated with W661 learnings.

---

## 4. Weak points addressed

1. **First outer dimension 141.** Confirmed the compiler and reference model correctly stride by 141 for a non-power-of-two outer dimension.
2. **Modulo-wrap signal.** The explicit `make_grid(32768)` check preserves the same wrap regression coverage as earlier waves even though the element count (9,024) is below the natural wrap point.
3. **Multi-line literal brace style.** Reused the W584/W660 multi-line brace style so the 6-D nested literal parses correctly.
4. **Simulator capacity.** At ≈0.275 MiBit the witness simulates in 17 cycles, well below the Icarus very-large-vector threshold.
5. **`assert_ne` Icarus emission gap.** Continued using `assert_eq` checks on changed elements, since the structural classifier accepts `assert_ne` but the simulation emitter does not lower it.

---

## 5. Scientific / technical background

- **IEEE Std 1800-2017** §7.4.1/7.4.3 — packed-array total width is the product of packed dimensions; ranges need not be powers of two.
- **Sutherland, “Synthesizable SystemVerilog”** — packed arrays and packed structs as synthesizable first-class objects.
- **Icarus Verilog Quirks / Extensions pages** — width handling and packed-array subset behavior.
- **Icarus issue #1134** — assertion failures with unpacked arrays of packed structs; t27 flattening avoids the trigger.
- **Icarus issue #1171** — freezes during elaboration of very large packed vectors; W661 stays far below the reported threshold.
- **Yosys docs / PR #4100 / issue #4653 / issue #2677** — multidimensional packed arrays supported, arrays of packed structs unsupported; t27 flattening avoids the gap.
- **cocotb PR #3608 / discussion #2933** — packed structs as whole signals; flat `LogicArray` for multidimensional packed arrays in the reference model.
- **Lutsig (CPP 2021)** — verified array-read lowering.
- **CIRCT `HWLegalizeModules.cpp` / SV dialect** — production packed-array scalarization.

---

## 6. L1–L7 compliance

| Law | Status | Evidence |
|-----|--------|----------|
| L1 TRACEABILITY | ✅ | This report and commits reference `Closes #1632`. |
| L2 GENERATION | ✅ | `gen/` untouched; all generated artifacts produced by `t27c`. |
| L3 PURITY | ✅ | Source identifiers ASCII/English; witness uses approved style. |
| L4 TESTABILITY | ✅ | Witness contains `bench` block with assertions and cocotb cross-check. |
| L5 IDENTITY | ✅ | No math/physics changes; φ invariants unaffected. |
| L6 CEILING | ✅ | No numeric SSOT files touched. |
| L7 UNITY | ✅ | No new `*.sh` on critical path; used `tri`/`t27c`. |

---

## 7. Next Wave Loop 662 cooperation variants

1. **Variant A — `[143][2]^6 Pt` module-scope var from a call with indexed signed writes.**  
   292,864-bit packed vector, 9,152 elements, non-power-of-two outer dimension 143, well under the 4-MiBit cliff. Continues the odd outer-dimension ladder. **Recommended.**

2. **Variant B — `[141][2]^6 Pt` bench-local (function-local) packed array var from a call with indexed signed writes.**  
   288,768-bit packed vector, 9,024 elements. Tests that the same non-p2 outer dimension works when the mutable `reg` is declared inside a bench/function rather than at module scope. Useful complement to the module-scope ladder.

3. **Variant C — `[141][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes.**  
   Stays at 0.275 MiBit and tests that control-flow guarded indexed writes on a packed `reg` are correctly elaborated and simulated (e.g. write only when a signed index exceeds a threshold). Useful follow-up to W590/W591.

---

## 8. Commit summary

- Feature commit: `feat(igla): Wave Loop 661 — module-scope [141][2]^6 Pt non-p2 AoS var from a call with indexed signed writes`  
  `Closes #1632`
- Tracking commit: records session log, commit count, plan, closeout report, seal, baseline, and experience update.

---

Phase complete: Verify
→ Phase 9: Learn
