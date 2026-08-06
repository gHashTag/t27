# FPGA/Wave Loop 656 Closeout — Issue #1627

**Date:** 2026-07-07  
**Branch:** `wave-loop-656`  
**Variant:** A — module-scope `[131][2]^6 Pt` non-power-of-two array-of-struct variable initialized from a function call, with indexed signed field writes and read-back.

---

## 1. Goal

Continue the module-scope packed array-of-struct (AoS) odd outer-dimension ladder to `131` and confirm that the t27 compiler and reference model handle a non-power-of-two outer dimension at 268,288 bits (≈0.256 MiBit) with no compiler or reference-model changes.

---

## 2. What was implemented

- Witness spec: `specs/scratch/w656_bench_module_131x2p6_aos_var_call_write.t27`
  - `pub struct Pt { x : i16, y : i16 }`
  - `pub fn make_grid(offset : u16) -> [131][2][2][2][2][2][2]Pt` returning an 8,384-element packed literal.
  - `pub const expected : [131][2][2][2][2][2][2]Pt = make_grid(0);`
  - `pub var dst : [131][2][2][2][2][2][2]Pt = make_grid(0);`
  - `test module_var_131x2p6_call_write`: initial-state equality, corner indexed reads, and an explicit `make_grid(32768)` modulo-wrap check.
  - `bench module_bench_131x2p6_call_write`: whole-array equality before writes, indexed reads, signed indexed field writes, read-back, frame-condition checks, and changed-element checks (avoiding `assert_ne`, which the Icarus simulation path does not emit).
- Generator: `scripts/gen_w656.py` (derived from `scripts/gen_w655.py`, `OUTER = 131`).
- Integration test: `accepts_w656_bench_module_131x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
- Seal: `.trinity/seals/scratch_w656_bench_module_131x2p6_aos_var_call_write.json`.
- Empty Icarus baseline: `.trinity/icarus-baselines/specs/scratch/w656_bench_module_131x2p6_aos_var_call_write.json`.

---

## 3. Verification gate results

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | ✅ green (626 warnings, all pre-existing) |
| `t27c parse specs/scratch/w656_bench_module_131x2p6_aos_var_call_write.t27` | ✅ PASS |
| `t27c icarus-lowerable --json ...` | ✅ `{ "lowerable": true }` |
| `t27c icarus-simulate ...` | ✅ PASSED (17 cycles) |
| `t27c icarus-cocotb ...` | ✅ reference-model OK |
| `t27c seal --save ...` | ✅ saved |
| `cargo test -p t27c --bin t27c` | ✅ 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | ✅ 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | ✅ 116 passed; 0 failed |

FROZEN_HASH remains unchanged at `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

---

## 4. Weak points investigated

1. **First outer dimension 131.** The compiler and reference model must index/stride by 131 at the outer dimension. Prior non-p2 witnesses (3, 5, 7, …, 129) strongly predicted success; the end-to-end module-scope witness confirms it.
2. **Modulo-wrap regression signal.** With only 8,384 elements, the offset-0 schedule `(2*e + offset) % 32768` never wraps (max raw 16,767). The test retains the explicit `make_grid(32768)` call to keep the wrap check equivalent to earlier waves.
3. **Multi-line literal brace style.** The 6-D nested literal continues to use the W584 multi-line style; the parser accepts it and generated Verilog is valid.
4. **Simulator capacity.** At 0.256 MiBit the witness elaborates and simulates in 17 cycles, far below the Icarus very-large-vector threshold.
5. **`assert_ne` gap.** The structural classifier accepts `assert_ne`, but `gen_verilog_test_stmt` only lowers `assert_eq`. The bench substitutes changed-element `assert_eq` checks, keeping all gates passing without compiler changes.

---

## 5. Scientific / technical references

- IEEE Std 1800-2017 — packed-array total width as product of dimensions; ranges need not be powers of two.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays and packed structs as synthesizable first-class objects.
- Icarus Verilog Quirks / Extensions pages — width handling and packed-array subset behavior.
- Icarus issue #1134 — assertion failures with unpacked arrays of packed structs; t27 flattening avoids the trigger.
- Icarus issue #1171 — freezes during elaboration of very large packed vectors; W656 stays far below the reported threshold.
- Yosys docs / PR #4100 / issue #4653 / issue #2677 — multidimensional packed arrays supported, arrays of packed structs unsupported; t27 flattening avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array scalarization.

---

## 6. Cooperation variants for Wave Loop 657

1. **Variant A — `[133][2]^6 Pt` module-scope var from a call with indexed signed writes.**  
   272,384-bit packed vector, 8,512 elements, non-power-of-two outer dimension 133. Continues the odd outer-dimension ladder well under the 4-MiBit cliff. **Recommended.**

2. **Variant B — `[131][2]^6 Pt` bench-local (function-local) packed array var from a call with indexed signed writes.**  
   268,288-bit packed vector, 8,384 elements. Tests that the same non-p2 outer dimension works when the mutable `reg` is declared inside a bench/function rather than at module scope. Useful complement to the module-scope ladder.

3. **Variant C — `[131][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes.**  
   Stays at 0.256 MiBit and tests that control-flow-guarded indexed writes on a packed `reg` are correctly elaborated and simulated (e.g., write only when a signed index exceeds a threshold). Useful follow-up to W590/W591.

---

## 7. Commits / artifacts

- Witness: `specs/scratch/w656_bench_module_131x2p6_aos_var_call_write.t27`
- Generator: `scripts/gen_w656.py`
- Integration test: `bootstrap/tests/icarus_lowerable.rs`
- Seal: `.trinity/seals/scratch_w656_bench_module_131x2p6_aos_var_call_write.json`
- Empty Icarus baseline: `.trinity/icarus-baselines/specs/scratch/w656_bench_module_131x2p6_aos_var_call_write.json`
- Plan: `.claude/plans/wave-loop-656.md`
- This closeout report: `docs/reports/FPGA_LOOP_CLOSEOUT_W656_2026-07-07.md`
- Updated tracking: `.trinity/current-issue.md`, `.trinity/experience.md`

Closes #1627.

φ² + 1/φ² = 3 | TRINITY
