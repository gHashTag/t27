# Wave Loop 641 Plan — `[101][2]^6 Pt` module-scope non-p2 AoS var from call

**Issue:** #1612  
**Branch:** `wave-loop-641`  
**Date:** 2026-07-07  
**Variant chosen:** A — `[101][2]^6 Pt` module-scope variable initialized from a function call, with indexed signed field writes and read-back.

---

## 1. Weak points investigated

| # | Weak point | Why it matters | Mitigation in W641 |
|---|------------|----------------|---------------------|
| 1 | **First outer dimension 101** | The compiler and reference model must stride by 101 at the outer dimension. Prior waves (3, 5, 7, ..., 97, 99) suggest the path is safe, but a dedicated module-scope witness is required for end-to-end proof. | Reuse the W640 module-scope lowerable style with outer dimension 101. |
| 2 | **Modulo-wrap regression signal** | 6,464 elements are below the natural wrap point of the i16 schedule `(2*e + offset) % 32768`; max raw value is 12,927, so offset 0 never wraps. | Keep the explicit `make_grid(32768)` assertion so the wrap path remains covered. |
| 3 | **Multi-line mega-literals** | A single-line 6-D nested literal risks parser truncation; the established style splits every `[2]^k Pt` level across lines. | Use the same W584/W605/W640 multi-line brace style with explicit `[101][2][2][2][2][2][2]Pt` annotations. |
| 4 | **Simulator capacity** | 206,848 bits (≈0.197 MiBit) is still far below the 4-MiBit cliff, but each dimension increase nudges resource use upward. | Stay well under the cliff; expect fast interactive simulation. |
| 5 | **Index correctness in tests** | Moving from 99 to 101 changes the last-row index (98→100) and the mid-row index (49→50). | Update all corner indices and expected values accordingly (last element `12926/12927`, mid element `6464/6465`). |
| 6 | **Spec size** | The witness grows to ~19,300 lines; ensure the literal remains syntactically valid and the file stays manageable. | Generate by extending W640 with two additional row blocks rather than rewriting the mega-literal from scratch. |
| 7 | **`assert_ne` simulation gap** | W640 documented that `assert_ne` is structurally lowerable but not emitted by the Icarus simulation path. | Continue using `assert_eq` checks on changed elements in the bench block. |

---

## 2. Scientific / technical background

- **IEEE Std 1800-2017** — packed-array total width is the product of dimensions; ranges need not be powers of two. W641 relies on this for the non-power-of-two outer dimension 101.
- **Sutherland, “Synthesizable SystemVerilog”** — packed arrays and packed structs are synthesizable first-class objects; t27 lowers them to flat vectors.
- **Icarus Verilog Quirks / Extensions** — Icarus handles packed arrays/structs in the flattened form t27 emits; the remaining trigger conditions (unpacked arrays of packed structs, very wide vectors) are avoided by design.
- **Icarus issue #1171** — reported freezes during elaboration of very large packed vectors; W641 at 0.197 MiBit stays far below the reported threshold.
- **Yosys docs / issues #2677, #4653, PR #4100** — multidimensional packed arrays are supported, arrays of packed structs are not; t27 flattening avoids the unsupported construct.
- **cocotb `LogicArray`** — the Python reference model treats the generated flat vector as a packed multidimensional array and computes element offsets row-major LSB-first.
- **Lutsig (CPP 2021)** — verified array-read lowering, relevant because W641 exercises indexed reads and writes through a non-power-of-two outer stride.
- **CIRCT `HWLegalizeModules.cpp` / SV dialect** — production compilers scalarize packed arrays; t27’s flattening strategy aligns with this industrial trend.

---

## 3. Decomposed tasks

1. **Branch setup** — create `wave-loop-641` from the W640 branch head and update `.trinity/current-issue.md` with chosen variant.
2. **Spec generation** — produce `specs/scratch/w641_bench_module_101x2p6_aos_var_call_write.t27` by extending the W640 witness to 101 rows, regenerating sequential element values, and updating corner indices in `test`/`bench`.
3. **Compiler health check** — `cargo build --release -p t27c`.
4. **Parse gate** — `t27c parse ...`.
5. **Icarus lowerable gate** — `t27c icarus-lowerable ...`.
6. **Integration test** — add `accepts_w641_bench_module_101x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
7. **Simulation gate** — `t27c icarus-simulate ...`.
8. **Reference gate** — `t27c icarus-cocotb ...`.
9. **Seal** — `t27c seal --save ...` and empty Icarus baseline.
10. **Closeout report** — `docs/reports/FPGA_LOOP_CLOSEOUT_W641_2026-07-07.md` with verification matrix, weak points, literature, and three W642 cooperation variants.
11. **Tracking commit** — commit W641 feature with `Closes #1612`, then commit hook-generated `.trinity/current_task/*` increment.
12. **Experience save** — append W641 learnings to `.trinity/experience.md` and `~/.claude/projects/-Users-playra-t27/memory/wave-loop-641.md`, update `MEMORY.md` index.

---

## 4. Sizing

- Outer dimension: 101 (non-power-of-two).
- Total elements: 101 × 2⁶ = 6,464.
- Packed vector width: 6,464 × 32 = 206,848 bits (≈0.197 MiBit).
- Spec lines: ~19,300 (multi-line brace style).

---

## 5. Success criteria

- `t27c parse` PASS.
- `t27c icarus-lowerable` reports `lowerable`.
- Integration test `accepts_w641_...` PASS.
- `t27c icarus-simulate` silent exit 0.
- `t27c icarus-cocotb` reports `reference-model OK`.
- Seal saved and FROZEN_HASH unchanged.
- `cargo test -p t27c --test icarus_lowerable` count increments to 101/0.
- No compiler or reference-model changes required.
