# Wave Loop 624 Plan — `[67][2]^6 Pt` module-scope non-p2 AoS var from call

**Issue:** #1595  
**Branch:** `wave-loop-624`  
**Date:** 2026-07-07  
**Variant chosen:** A — `[67][2]^6 Pt` module-scope variable initialized from a function call, with indexed signed field writes and read-back.

---

## 1. Weak points investigated

| # | Weak point | Why it matters | Mitigation in W624 |
|---|------------|----------------|---------------------|
| 1 | **First outer dimension 67** | The compiler and reference model must stride by 67 at the outer dimension. Prior waves (3, 5, 7, ..., 65) suggest the path is safe, but a dedicated module-scope witness is required for end-to-end proof. | Reuse the W623 module-scope lowerable style with outer dimension 67. |
| 2 | **Modulo-wrap regression signal** | 4,288 elements are below the natural wrap point of the i16 schedule `(2*e + offset) % 32768`; max raw value is 8,575, so offset 0 never wraps. | Keep the explicit `make_grid(32768)` assertion so the wrap path remains covered. |
| 3 | **Multi-line mega-literals** | A single-line 6-D nested literal risks parser truncation; the established style splits every `[2]^k Pt` level across lines. | Use the same W584/W605/W622 multi-line brace style. |
| 4 | **Simulator capacity** | 137,216 bits (≈0.130 MiBit) is still far below the 4-MiBit cliff, but each dimension increase nudges resource use upward. | Stay well under the cliff; expect fast interactive simulation. |
| 5 | **Index correctness in tests** | Moving from 65 to 67 changes the last-row index (64→66) and the mid-row index (32→33). | Update all corner indices and expected values accordingly (last element `8574/8575`, mid element `4288/4289`). |
| 6 | **Spec size** | The witness grows to ~15,000 lines; ensure the literal remains syntactically valid and the file stays manageable. | Generate by extending W623 with two additional row blocks rather than rewriting from scratch. |

---

## 2. Scientific / technical background

- **IEEE Std 1800-2017** — packed-array total width is the product of dimensions; ranges need not be powers of two. W624 relies on this for the non-power-of-two outer dimension 67.
- **Sutherland, “Synthesizable SystemVerilog”** — packed arrays and packed structs are synthesizable first-class objects; t27 lowers them to flat vectors.
- **Icarus Verilog Quirks / Extensions** — Icarus handles packed arrays/structs in the flattened form t27 emits; the remaining trigger conditions (unpacked arrays of packed structs, very wide vectors) are avoided by design.
- **Icarus issue #1171** — reported freezes during elaboration of very large packed vectors; W624 at 0.130 MiBit stays far below the reported threshold.
- **Yosys docs / issues #2677, #4653, PR #4100** — multidimensional packed arrays are supported, arrays of packed structs are not; t27 flattening avoids the unsupported construct.
- **cocotb `LogicArray`** — the Python reference model treats the generated flat vector as a packed multidimensional array and computes element offsets row-major LSB-first.
- **Lutsig (CPP 2021)** — verified array-read lowering, relevant because W624 exercises indexed reads and writes through a non-power-of-two outer stride.
- **CIRCT `HWLegalizeModules.cpp` / SV dialect** — production compilers scalarize packed arrays; t27’s flattening strategy aligns with this industrial trend.

---

## 3. Decomposed tasks

1. **Branch setup** — create `wave-loop-624` from the W623 branch head and update `.trinity/current-issue.md` with chosen variant.
2. **Spec generation** — produce `specs/scratch/w624_bench_module_67x2p6_aos_var_call_write.t27` by extending the W623 witness to 67 rows, regenerating sequential element values, and updating corner indices in `test`/`bench`.
3. **Compiler health check** — `cargo build --release -p t27c`.
4. **Parse gate** — `t27c parse ...`.
5. **Icarus lowerable gate** — `t27c icarus-lowerable ...`.
6. **Integration test** — add `accepts_w624_bench_module_67x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
7. **Simulation gate** — `t27c icarus-simulate ...`.
8. **Reference gate** — `t27c icarus-cocotb ...`.
9. **Seal** — `t27c seal --save ...` and empty Icarus baseline.
10. **Closeout report** — `docs/reports/FPGA_LOOP_CLOSEOUT_W624_2026-07-07.md` with verification matrix, weak points, literature, and three W625 cooperation variants.
11. **Tracking commit** — commit W624 feature with `Closes #1595`, then commit hook-generated `.trinity/current_task/*` increment.
12. **Experience save** — append W624 learnings to `.trinity/experience.md` and `~/.claude/projects/-Users-playra-t27/memory/wave-loop-624.md`, update `MEMORY.md` index.

---

## 4. Sizing

- Outer dimension: 67 (non-power-of-two).
- Total elements: 67 × 2⁶ = 4,288.
- Packed vector width: 4,288 × 32 = 137,216 bits (≈0.130 MiBit).
- Spec lines: ~15,000 (multi-line brace style).

---

## 5. Success criteria

- `t27c parse` PASS.
- `t27c icarus-lowerable` reports `lowerable`.
- Integration test `accepts_w624_...` PASS.
- `t27c icarus-simulate` silent exit 0.
- `t27c icarus-cocotb` reports `reference-model OK`.
- Seal saved and FROZEN_HASH unchanged.
- `cargo test -p t27c --test icarus_lowerable` count increments to 84/0.
- No compiler or reference-model changes required.
