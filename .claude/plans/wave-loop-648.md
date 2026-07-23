# Wave Loop 648 Plan — `[115][2]^6 Pt` module-scope non-p2 AoS var from call

**Issue:** #1619  
**Branch:** `wave-loop-648`  
**Date:** 2026-07-07  
**Variant chosen:** A — `[115][2]^6 Pt` module-scope variable initialized from a function call, with indexed signed field writes and read-back.

---

## 1. Weak points investigated

| # | Weak point | Why it matters | Mitigation in W648 |
|---|------------|----------------|---------------------|
| 1 | **First outer dimension 115** | The compiler and reference model must stride by 115 at the outer dimension. Prior waves (3, 5, 7, ..., 111, 113) suggest the path is safe, but a dedicated module-scope witness is required for end-to-end proof. | Reuse the W647 module-scope lowerable style with outer dimension 115. |
| 2 | **Modulo-wrap regression signal** | 7,360 elements are below the natural wrap point of the i16 schedule `(2*e + offset) % 32768`; max raw value is 14,719, so offset 0 never wraps. | Keep the explicit `make_grid(32768)` assertion so the wrap path remains covered. |
| 3 | **Multi-line mega-literals** | A single-line 6-D nested literal risks parser truncation; the established style splits every `[2]^k Pt` level across lines. | Use the same W584/W605/W647 multi-line brace style with explicit `[115][2][2][2][2][2][2]Pt` annotations. |
| 4 | **Simulator capacity** | 236,544 bits (≈0.225 MiBit) is still far below the 4-MiBit cliff, but each dimension increase nudges resource use upward. | Stay well under the cliff; expect fast interactive simulation. |
| 5 | **Index correctness in tests** | Moving from 113 to 115 changes the last-row index (112→114) and the mid-row index (56→57). | Update all corner indices and expected values accordingly (last element `14718/14719`, mid element `7360/7361`). |
| 6 | **Spec size** | The witness grows to ~21,900 lines; ensure the literal remains syntactically valid and the file stays manageable. | Generate by extending W647 with two additional row blocks rather than rewriting the mega-literal from scratch. |
| 7 | **`assert_ne` simulation gap** | W647 documented that `assert_ne` is structurally lowerable but not emitted by the Icarus simulation path. | Continue using `assert_eq` checks on changed elements in the bench block. |

---

## 2. Scientific / technical background

- **IEEE Std 1800-2017** — packed-array total width is the product of dimensions; ranges need not be powers of two. W648 relies on this for the non-power-of-two outer dimension 115.
- **Sutherland, “Synthesizable SystemVerilog”** — packed arrays and packed structs are synthesizable first-class objects; t27 lowers them to flat vectors.
- **Icarus Verilog Quirks / Extensions** — Icarus handles packed arrays/structs in the flattened form t27 emits; the remaining trigger conditions (unpacked arrays of packed structs, very wide vectors) are avoided by design.
- **Icarus issue #1171** — reported freezes during elaboration of very large packed vectors; W648 at 0.225 MiBit stays far below the reported threshold.
- **Yosys docs / issues #2677, #4653, PR #4100** — multidimensional packed arrays are supported, arrays of packed structs are not; t27 flattening avoids the unsupported construct.
- **cocotb `LogicArray`** — the Python reference model treats the generated flat vector as a packed multidimensional array and computes element offsets row-major LSB-first.
- **Lutsig (CPP 2021)** — verified array-read lowering, relevant because W648 exercises indexed reads and writes through a non-power-of-two outer stride.
- **CIRCT `HWLegalizeModules.cpp` / SV dialect** — production compilers scalarize packed arrays; t27’s flattening strategy aligns with this industrial trend.

---

## 3. Decomposed tasks

1. **Branch setup** — create `wave-loop-648` from the W647 branch head and update `.trinity/current-issue.md` with chosen variant.
2. **Spec generation** — produce `specs/scratch/w648_bench_module_115x2p6_aos_var_call_write.t27` by extending the W647 witness to 115 rows, regenerating sequential element values, and updating corner indices in `test`/`bench`.
3. **Compiler health check** — `cargo build --release -p t27c`.
4. **Parse gate** — `t27c parse ...`.
5. **Icarus lowerable gate** — `t27c icarus-lowerable ...`.
6. **Integration test** — add `accepts_w648_bench_module_115x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
7. **Simulation gate** — `t27c icarus-simulate ...`.
8. **Reference gate** — `t27c icarus-cocotb ...`.
9. **Seal** — `t27c seal --save ...` and empty Icarus baseline.
10. **Closeout report** — `docs/reports/FPGA_LOOP_CLOSEOUT_W648_2026-07-07.md` with verification matrix, weak points, literature, and three W649 cooperation variants.
11. **Tracking commit** — commit W648 feature with `Closes #1619`, then commit hook-generated `.trinity/current_task/*` increment.
12. **Experience save** — append W648 learnings to `.trinity/experience.md` and `~/.claude/projects/-Users-playra-t27/memory/wave-loop-648.md`, update `MEMORY.md` index.

---

## 4. Sizing

- Outer dimension: 115 (non-power-of-two).
- Total elements: 115 × 2⁶ = 7,360.
- Packed vector width: 7,360 × 32 = 236,544 bits (≈0.225 MiBit).
- Spec lines: ~21,900 (multi-line brace style).

---

## 5. Success criteria

- `t27c parse` PASS.
- `t27c icarus-lowerable` reports `lowerable`.
- Integration test `accepts_w648_...` PASS.
- `t27c icarus-simulate` silent exit 0.
- `t27c icarus-cocotb` reports `reference-model OK`.
- Seal saved and FROZEN_HASH unchanged.
- `cargo test -p t27c --test icarus_lowerable` count increments to 108/0.
- No compiler or reference-model changes required.
