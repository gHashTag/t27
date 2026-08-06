# Wave Loop 639 Plan — `[97][2]^6 Pt` module-scope non-p2 AoS var from call

**Issue:** #1610  
**Branch:** `wave-loop-639`  
**Date:** 2026-07-07  
**Variant chosen:** A — `[97][2]^6 Pt` module-scope variable initialized from a function call, with indexed signed field writes and read-back.

---

## 1. Weak points investigated

| # | Weak point | Why it matters | Mitigation in W639 |
|---|------------|----------------|---------------------|
| 1 | **First outer dimension 97** | The compiler and reference model must stride by 97 at the outer dimension. Prior waves (3, 5, 7, ..., 93, 95) suggest the path is safe, but a dedicated module-scope witness is required for end-to-end proof. | Reuse the W638 module-scope lowerable style with outer dimension 97. |
| 2 | **Modulo-wrap regression signal** | 6,208 elements are below the natural wrap point of the i16 schedule `(2*e + offset) % 32768`; max raw value is 12,415, so offset 0 never wraps. | Keep the explicit `make_grid(32768)` assertion so the wrap path remains covered. |
| 3 | **Multi-line mega-literals** | A single-line 6-D nested literal risks parser truncation; the established style splits every `[2]^k Pt` level across lines. | Use the same W584/W605/W638 multi-line brace style with explicit `[97][2][2][2][2][2][2]Pt` annotations. |
| 4 | **Simulator capacity** | 198,656 bits (≈0.189 MiBit) is still far below the 4-MiBit cliff, but each dimension increase nudges resource use upward. | Stay well under the cliff; expect fast interactive simulation. |
| 5 | **Index correctness in tests** | Moving from 95 to 97 changes the last-row index (94→96) and the mid-row index (47→48). | Update all corner indices and expected values accordingly (last element `12414/12415`, mid element `6208/6209`). |
| 6 | **Spec size** | The witness grows to ~18,500 lines; ensure the literal remains syntactically valid and the file stays manageable. | Generate by extending W638 with two additional row blocks rather than rewriting the mega-literal from scratch. |
| 7 | **`assert_ne` simulation gap** | W638 documented that `assert_ne` is structurally lowerable but not emitted by the Icarus simulation path. | Continue using `assert_eq` checks on changed elements in the bench block. |

---

## 2. Scientific / technical background

- **IEEE Std 1800-2017** — packed-array total width is the product of dimensions; ranges need not be powers of two. W639 relies on this for the non-power-of-two outer dimension 97.
- **Sutherland, “Synthesizable SystemVerilog”** — packed arrays and packed structs are synthesizable first-class objects; t27 lowers them to flat vectors.
- **Icarus Verilog Quirks / Extensions** — Icarus handles packed arrays/structs in the flattened form t27 emits; the remaining trigger conditions (unpacked arrays of packed structs, very wide vectors) are avoided by design.
- **Icarus issue #1171** — reported freezes during elaboration of very large packed vectors; W639 at 0.189 MiBit stays far below the reported threshold.
- **Yosys docs / issues #2677, #4653, PR #4100** — multidimensional packed arrays are supported, arrays of packed structs are not; t27 flattening avoids the unsupported construct.
- **cocotb `LogicArray`** — the Python reference model treats the generated flat vector as a packed multidimensional array and computes element offsets row-major LSB-first.
- **Lutsig (CPP 2021)** — verified array-read lowering, relevant because W639 exercises indexed reads and writes through a non-power-of-two outer stride.
- **CIRCT `HWLegalizeModules.cpp` / SV dialect** — production compilers scalarize packed arrays; t27’s flattening strategy aligns with this industrial trend.

---

## 3. Decomposed tasks

1. **Branch setup** — create `wave-loop-639` from the W638 branch head and update `.trinity/current-issue.md` with chosen variant.
2. **Spec generation** — produce `specs/scratch/w639_bench_module_97x2p6_aos_var_call_write.t27` by extending the W638 witness to 97 rows, regenerating sequential element values, and updating corner indices in `test`/`bench`.
3. **Compiler health check** — `cargo build --release -p t27c`.
4. **Parse gate** — `t27c parse ...`.
5. **Icarus lowerable gate** — `t27c icarus-lowerable ...`.
6. **Integration test** — add `accepts_w639_bench_module_97x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
7. **Simulation gate** — `t27c icarus-simulate ...`.
8. **Reference gate** — `t27c icarus-cocotb ...`.
9. **Seal** — `t27c seal --save ...` and empty Icarus baseline.
10. **Closeout report** — `docs/reports/FPGA_LOOP_CLOSEOUT_W639_2026-07-07.md` with verification matrix, weak points, literature, and three W640 cooperation variants.
11. **Tracking commit** — commit W639 feature with `Closes #1610`, then commit hook-generated `.trinity/current_task/*` increment.
12. **Experience save** — append W639 learnings to `.trinity/experience.md` and `~/.claude/projects/-Users-playra-t27/memory/wave-loop-639.md`, update `MEMORY.md` index.

---

## 4. Sizing

- Outer dimension: 97 (non-power-of-two).
- Total elements: 97 × 2⁶ = 6,208.
- Packed vector width: 6,208 × 32 = 198,656 bits (≈0.189 MiBit).
- Spec lines: ~18,500 (multi-line brace style).

---

## 5. Success criteria

- `t27c parse` PASS.
- `t27c icarus-lowerable` reports `lowerable`.
- Integration test `accepts_w639_...` PASS.
- `t27c icarus-simulate` silent exit 0.
- `t27c icarus-cocotb` reports `reference-model OK`.
- Seal saved and FROZEN_HASH unchanged.
- `cargo test -p t27c --test icarus_lowerable` count increments to 99/0.
- No compiler or reference-model changes required.
