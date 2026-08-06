# Wave Loop 652 Plan — module-scope `[123][2]^6 Pt` non-p2 AoS variable

**Issue:** #1623  
**Branch:** `wave-loop-652`  
**Variant:** A — module-scope `[123][2]^6 Pt` non-power-of-two array-of-struct
variable initialized from a function call, with indexed signed field writes and
read-back.

**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Objective

Extend the module-scope packed array-of-struct odd outer-dimension ladder from
121 (W651) to 123. Confirm that a module-level mutable `reg` of type
`[123][2]^6 Pt` can be initialized from a function call and exercised with
signed indexed reads and writes without compiler or reference-model changes.

---

## 2. Sizing

- Outer dimension: 123 (odd non-power-of-two).
- Inner dimensions: `[2]^6` = 64.
- Total scalar elements: 123 × 64 = **7,872**.
- Packed vector width: 7,872 × 32 bits = **252,096 bits** ≈ **0.240 MiBit**.
- Well below the observed 4-MiBit Icarus/Yosys elaboration cliff.

---

## 3. Weak points to investigate

| # | Weak point | Hypothesis | Verification |
|---|------------|------------|--------------|
| 1 | **Outer dimension 123** | Compiler/reference model correctly stride by 123 at the outer dimension. | Direct `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`. |
| 2 | **Modulo-wrap regression signal** | With 7,872 elements the offset-0 schedule still never wraps; keep explicit `make_grid(32768)` to exercise the wrap path. | `test` block equality check against `make_grid(32768)`. |
| 3 | **Multi-line mega-literals** | W584-style multi-line brace splitting remains necessary and valid for the 6-D literal inside the 123× outer shape. | Parser and structural classifier must accept the spec. |
| 4 | **Simulator capacity** | 0.240 MiBit is still far below the 4-MiBit cliff; simulation should complete in ~17 cycles. | `t27c icarus-simulate` timing and PASS verdict. |
| 5 | **Index correctness in tests** | Mid-row values must account for inner `[2]^6` layout; reuse W632 corrected formula. | Simulation must not assert. |
| 6 | **Spec size / generator scaling** | Generator should extend W651 by two row blocks rather than rewriting the literal. | File size ~536 KB, ~23,500 lines. |
| 7 | **`assert_ne` simulation gap** | `assert_ne` is accepted by `icarus-lowerable` but not emitted by the Icarus simulation path; continue using `assert_eq` on changed elements. | No `assert_ne` in simulated bench. |

---

## 4. Scientific / technical background

- IEEE Std 1800-2017 — packed-array total width is the product of dimensions;
  ranges need not be powers of two.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are
  synthesizable first-class objects.
- Icarus Verilog Quirks / Extensions pages — width handling and packed-array
  subset behavior.
- Icarus issue #1134 — assertion failures with unpacked arrays of packed
  structs; t27 flattening avoids the trigger.
- Icarus issue #1171 — freezes during elaboration of very large packed vectors;
  W652 stays far below the reported threshold.
- Yosys docs / PR #4100 / issue #4653 / issue #2677 — multidimensional packed
  arrays supported, arrays of packed structs still unsupported; t27 flattening
  avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

---

## 5. Decomposed tasks

1. **Issue / context**
   - Update `.trinity/current-issue.md` for W652 (#1623, branch `wave-loop-652`, Variant A).
   - Carry forward W653 cooperation variants.
2. **Spec generation**
   - Copy `scripts/gen_w651.py` to `scripts/gen_w652.py`.
   - Set `OUTER = 123`; adjust module/test/bench names and expected values.
   - Generate `specs/scratch/w652_bench_module_123x2p6_aos_var_call_write.t27`.
3. **Test coverage**
   - Add integration test `accepts_w652_bench_module_123x2p6_aos_var_call_write` in
     `bootstrap/tests/icarus_lowerable.rs`.
4. **Verification gates**
   - `cargo build --release -p t27c`
   - `t27c parse`
   - `t27c icarus-lowerable --json`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test -p t27c --bin t27c`
   - `cargo test -p tri`
   - `cargo test -p t27c --test icarus_lowerable`
5. **Closeout**
   - Write `docs/reports/FPGA_LOOP_CLOSEOUT_W652_2026-07-07.md`.
   - Update `.trinity/experience.md`.
   - Persist memory: `~/.claude/projects/-Users-playra-t27/memory/wave-loop-652.md` + `MEMORY.md` index.
6. **Land**
   - Feature commit with `Closes #1623`.
   - Tracking commit for session log / commit count.

---

## 6. Success criteria

- `t27c parse` W652 PASS.
- `t27c icarus-lowerable` W652 `lowerable: true`.
- `t27c icarus-simulate` W652 PASSED (≈17 cycles).
- `t27c icarus-cocotb` W652 `reference-model OK`.
- `t27c seal --save` W652 saved with stable `spec_hash`.
- `cargo test -p t27c --test icarus_lowerable` increases to **112/0**.
- `cargo test -p t27c --bin t27c` remains 1494/0/2.
- `cargo test -p tri` remains 78/0.
- `bootstrap/stage0/FROZEN_HASH` unchanged.
- No compiler or reference-model source changes.

---

## 7. Risk register

| Risk | Mitigation |
|------|------------|
| Simulation time creep | 0.240 MiBit is still ~16× below the 4-MiBit cliff; monitor wall-clock. |
| Mid-row expected value off-by-one | Use explicit element index `MID_IDX*64 + 32`; verify with simulation. |
| File too large for git hooks | Previous ~528 KB specs pass L3 PURITY; new ~536 KB should too. |

---

## 8. Next Wave Loop 653 cooperation variants (preliminary)

### Variant A — `[125][2]^6 Pt` module-scope var from a call with indexed signed writes *(Recommended)*

- Outer dimension 125 (next odd non-power-of-two).
- 8,000 elements, 256,000-bit packed vector (~0.244 MiBit).
- Safest continuation of the established ladder.

### Variant B — `[123][2]^6 Pt` bench-local (function-local) packed array var from a call with indexed signed writes

- Keeps the 252,096-bit vector and moves the mutable `reg` into a `bench` or
  function scope.
- Complements the module-scope ladder with local-scope coverage.

### Variant C — `[123][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes

- Keeps the 0.240 MiBit vector and adds a conditional `if` branch around indexed
  signed field writes.
- Tests control-flow guarded indexed writes on a packed `reg`.
