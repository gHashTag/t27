# Wave Loop 599 Plan — `[17][2]^9 Pt` module-scope AoS var from call

**Issue:** #1570  
**Branch:** `wave-loop-599`  
**Previous:** Wave Loop 598 (#1569, `wave-loop-598`)  
**Date:** 2026-07-07  
**Estimated complexity:** Low — zero compiler/reference-model changes expected.

## Goal

Validate a module-scope, mutable, packed array-of-scalar-struct variable with a
non-power-of-two outer dimension of **17**, initialized from a function call, and
exercised with indexed signed field writes and read-back.

## Variant rationale

- **Variant A (chosen):** `[17][2]^9 Pt` — continues the odd outer-dimension
  ladder (3 → 5 → 7 → 9 → 11 → 13 → 15 → 17). The 0.27 MiBit total is small
  enough for fast direct simulation while still proving the compiler's generic
  packed-AoS paths handle outer stride 17 end-to-end.
- **Variant B (rejected):** `[2]^18 Pt` — crosses the 4-MiBit cliff; risks
  Icarus/Yosys capacity or runaway parse time without chunked-literal support.
- **Variant C (deferred):** `[17][2]^9 Pt` with conditional whole-array
  reassignment — useful, but the priority is extending the non-p2 outer ladder.

## Sizing

For `[17][2]^9 Pt`:

- Elements = `17 × 2^9 = 8,704`.
- Bits = `8,704 × 32 = 278,528`.
- Expected witness file ≈ 1.5 MB (multi-line brace style).

## Weak-point analysis

| Weak point | Why it matters | Mitigation in this wave |
|------------|----------------|---------------------------|
| Outer dimension 17 untested at module scope | Stride-17 multiplication and row-major flattening must be correct in compiler, simulator, and reference model. | Direct Icarus simulation + cocotb reference-model cross-check. |
| Element count below the modulo-wrap point | With 8,704 elements the offset-0 schedule `(2*e + offset) % 32768` never exceeds `i16` range (`max raw = 17,407`). Earlier waves used `e_wrap = 16384` to assert wrap behavior. | Add an explicit shifted call `make_grid(32768)` and assert its first/last elements equal `(offset + raw) % 32768`, proving the reference model and compiler both evaluate modulo correctly. |
| Single-line mega-literal parser truncation | Extreme-rank literals parsed silently but dropped trailing declarations in earlier waves. | Mandatory multi-line W584 brace style for the 9-D inner literal. |
| Full batch sweep blocked by unrelated giant specs | `./scripts/tri test --fast` Phase 1 Parse can stall on unrelated 4-MiBit specs. | Rely on direct Icarus/cocotb gates and document batch status. |

## Scientific / technical references

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array width as product of dimensions.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs as
  first-class synthesizable objects.
- Icarus Verilog Quirks / Extensions pages — packed-array subset behavior.
- Icarus issue #1134 — unpacked arrays of packed structs cause assertion
  failures; t27 flattening avoids the trigger.
- Icarus issue #1171 — large packed vectors can freeze elaboration; W599 stays
  far below the reported threshold.
- Yosys docs / PR #4100 / issue #4653 / issue #2677 — multidimensional packed
  arrays supported, arrays of packed structs unsupported; t27 flattening
  avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals and
  flat `LogicArray` for multidimensional packed arrays.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

## Decomposition

1. **Spec generation**
   - Adapt `/tmp/gen_w598.py` to `DIMS = [17] + [2] * 9`.
   - Use the W584 multi-line brace style.
   - Leaf value schedule: `x = (2*e + offset) % 32768`,
     `y = (2*e + offset + 1) % 32768`.
   - Add explicit modulo-wrap assertion with `make_grid(32768)`.
   - Emit `pub const expected`, `pub var dst = make_grid(0)`, test and bench.

2. **Compiler / integration test**
   - Add `accepts_w599_bench_module_17x2p9_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.
   - Create empty Icarus baseline JSON.

3. **Seal and verify**
   - `cargo build --release -p t27c`.
   - `cargo test -p t27c --test icarus_lowerable`.
   - `t27c seal --save` on the witness.
   - Direct `t27c icarus-simulate` and `t27c icarus-cocotb`.

4. **Closeout**
   - `docs/reports/FPGA_LOOP_CLOSEOUT_W599_2026-07-07.md`.
   - Update `.trinity/experience.md`.
   - Persist memory `wave-loop-599.md` and `MEMORY.md` index update.
   - Commit with `Closes #1570`.

## Success criteria

- Witness parses, lowers, simulates, and passes cocotb reference model.
- `icarus_lowerable` test passes.
- No seal mismatches; FROZEN_HASH unchanged.
- Zero compiler or reference-model changes.
