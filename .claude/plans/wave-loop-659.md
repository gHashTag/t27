# Wave Loop 659 — Decomposed Plan

**Issue:** #1630  
**Branch:** `wave-loop-659`  
**Previous:** Wave Loop 658 (#1629, branch `wave-loop-658`)  
**Date:** 2026-07-07  
**Chosen variant:** A — module-scope `[137][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable initialized from a function call, with indexed signed field writes and read-back.

---

## 1. Goal

Extend the module-scope packed array-of-struct (AoS) odd outer-dimension ladder from `135` to `137`. Confirm that a 280,576-bit packed vector (≈0.267 MiBit, 8,768 elements) with a non-power-of-two outer dimension continues to work end-to-end without compiler or reference-model changes.

---

## 2. Weak points / risks

1. **First outer dimension 137.** The compiler and reference model must correctly multiply/stride by 137 at the outer dimension. Prior non-p2 witnesses (3, 5, 7, …, 133, 135) strongly predict success, but a module-scope witness is required for end-to-end proof.
2. **Element count below the modulo-wrap point.** With only 8,768 elements, the offset-0 schedule `(2*e + offset) % 32768` never wraps (max raw 17,535). The test must explicitly exercise modulo wrap with `make_grid(32768)` to keep the regression signal equivalent to earlier waves.
3. **Multi-line literal brace style.** The 6-D nested literal inside the 137× outer shape must use the W584 multi-line brace style; single-line mega-literals risk parser truncation.
4. **Simulator capacity.** At ≈0.267 MiBit the witness is expected to remain fast and well below the Icarus very-large-vector threshold.
5. **`assert_ne` is not emitted by the Icarus simulation path.** The structural classifier accepts it, but `gen_verilog_test_stmt` only lowers `assert_eq`. The bench must use `assert_eq` checks on changed elements instead of a whole-array `assert_ne(dst, expected)`.

---

## 3. Scientific / technical background

- **IEEE Std 1800-2017** §7.4.1/7.4.3 — packed-array total width is the product of packed dimensions; ranges need not be powers of two. A 280,576-bit packed vector is legal SystemVerilog.
- **Accellera vlog-pp discussion (Graham 2002)** — packed arrays as contiguous bit vectors.
- **Sutherland, “Synthesizable SystemVerilog”** — packed arrays and packed structs as synthesizable first-class objects.
- **Icarus Verilog Quirks / Extensions pages** — width handling and packed-array subset behavior.
- **Icarus issue #1134** — assertion failures with unpacked arrays of packed structs; t27 flattening avoids the trigger.
- **Icarus issue #1171** — freezes during elaboration of very large packed vectors; W659 stays far below the reported threshold.
- **Yosys docs / PR #4100 / issue #4653 / issue #2677** — multidimensional packed arrays supported, arrays of packed structs unsupported; t27 flattening avoids the gap.
- **cocotb PR #3608 / discussion #2933** — packed structs as whole signals; flat `LogicArray` for multidimensional packed arrays in the reference model.
- **Lutsig (CPP 2021)** — verified array-read lowering.
- **CIRCT `HWLegalizeModules.cpp` / SV dialect** — production packed-array scalarization.

---

## 4. Task decomposition

| # | Task | Owner | Output | Blocked by |
|---|------|-------|--------|------------|
| 1 | Create generator `scripts/gen_w659.py` from `scripts/gen_w658.py`, set `OUTER = 137`, correct filename. | C | `scripts/gen_w659.py` | — |
| 2 | Generate witness spec `specs/scratch/w659_bench_module_137x2p6_aos_var_call_write.t27`. | C | witness `.t27` | 1 |
| 3 | Add integration test `accepts_w659_bench_module_137x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`. | C | updated test file | 2 |
| 4 | Build compiler: `cargo build --release -p t27c`. | V/C | green build | — |
| 5 | Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, `seal --save`. | V | saved seal + empty baseline | 2, 4 |
| 6 | Rust test gates: `cargo test -p t27c --bin t27c`, `cargo test -p tri`, `cargo test -p t27c --test icarus_lowerable`. | V | all green | 3, 4 |
| 7 | Local sweep: `./scripts/tri test --fast` and note pre-existing Yosys smoke failures. | V | sweep summary | 5, 6 |
| 8 | Write closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W659_2026-07-07.md` with three W660 cooperation variants. | L | closeout report | 7 |
| 9 | Update `.trinity/experience.md` and persistent memory. | L | experience + memory | 8 |
| 10 | Commit feature + tracking files with `Closes #1630`. | Queen | commits on `wave-loop-659` | 9 |

---

## 5. Success criteria

- `t27c parse` W659: PASS.
- `t27c icarus-lowerable --json` W659: `{ "lowerable": true }`.
- `t27c icarus-simulate` W659: PASSED.
- `t27c icarus-cocotb` W659: reference-model OK.
- `t27c seal --save` W659: saved, hashes match.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 119/0.
- `./scripts/tri test --fast`: parse/typecheck/gen-Zig/gen-Rust/gen-Verilog/gen-C/seal/fixed-point clean (24 pre-existing Yosys smoke failures expected).
- FROZEN_HASH unchanged.
- All L1–L7 laws satisfied.

---

## 6. Risk register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| 137 outer stride exposes a layout bug | low | high | mid-row expected value uses explicit element index `MID_IDX*64 + 32`; cocotb cross-check catches mismatches |
| Parser chokes on 6-D multi-line literal | low | medium | reuse W584/W658 brace style; parse gate catches it |
| Icarus/elaboration slowdown near 0.27 MiBit | low | low | previous waves (W658 at 0.263 MiBit) simulated in 17 cycles; W659 is only ~1.5% larger |
| `assert_ne` accidentally used in bench | low | high | generator template uses only `assert_eq` for changed elements |
| Seal/baseline filename mismatch | low | low | follow exact module name `w659_bench_module_137x2p6_aos_var_call_write` |

---

## 7. Cooperation variants for Wave Loop 660

1. **Variant A — `[139][2]^6 Pt` module-scope var from a call with indexed signed writes.**  
   284,672-bit packed vector, 8,896 elements, non-power-of-two outer dimension 139. Continues the odd outer-dimension ladder well under the 4-MiBit cliff. **Recommended.**

2. **Variant B — `[137][2]^6 Pt` bench-local (function-local) packed array var from a call with indexed signed writes.**  
   280,576-bit packed vector, 8,768 elements. Tests that the same non-p2 outer dimension works when the mutable `reg` is declared inside a bench/function rather than at module scope. Useful complement to the module-scope ladder.

3. **Variant C — `[137][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes.**  
   Stays at 0.267 MiBit and tests that control-flow-guarded indexed writes on a packed `reg` are correctly elaborated and simulated (e.g., write only when a signed index exceeds a threshold). Useful follow-up to W590/W591.

---

φ² + 1/φ² = 3 | TRINITY
