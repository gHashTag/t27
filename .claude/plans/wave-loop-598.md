# Wave Loop 598 Plan — `[15][2]^10 Pt` module-scope AoS var from call

**Issue:** #1569  
**Branch:** `wave-loop-598`  
**Previous:** Wave Loop 597 (#1568, `wave-loop-597`)  
**Date:** 2026-07-07  
**Estimated complexity:** Low — zero compiler/reference-model changes expected.

## Goal

Validate a module-scope, mutable, packed array-of-scalar-struct variable with a
non-power-of-two outer dimension of **15**, initialized from a function call, and
exercised with indexed signed field writes and read-back.

## Variant rationale

- **Variant A (chosen):** `[15][2]^10 Pt` — continues the odd outer-dimension
  ladder (3 → 5 → 7 → 9 → 11 → 13 → 15). The 0.47 MiBit total is small enough
  for fast direct simulation while still proving the compiler's generic
  packed-AoS paths handle outer stride 15 end-to-end.
- **Variant B (rejected):** `[2]^18 Pt` — crosses the 4-MiBit cliff; risks
  Icarus/Yosys capacity or runaway parse time without chunked-literal support.
- **Variant C (deferred):** `[15][2]^10 Pt` with conditional whole-array
  reassignment — useful, but the priority is extending the non-p2 outer ladder.

## Sizing

For `[15][2]^10 Pt`:

- Elements = `15 × 2^10 = 15,360`.
- Bits = `15,360 × 32 = 491,520`.
- Expected witness file ≈ 2.8 MB (multi-line brace style).

## Weak-point analysis

| Weak point | Why it matters | Mitigation in this wave |
|------------|----------------|---------------------------|
| Outer dimension 15 untested at module scope | Stride-15 multiplication and row-major flattening must be correct in compiler, simulator, and reference model. | Direct Icarus simulation + cocotb reference-model cross-check. |
| Element count below the modulo-wrap point | With 15,360 elements the offset-0 schedule never exceeds `i16` range, so `% 32768` is a no-op at runtime. Earlier waves used `e_wrap = 16384` to assert wrap behavior. | Add an explicit shifted call `make_grid(32768)` and assert its first element equals `(0 + 32768) % 32768 = 0`, proving the reference model and compiler both evaluate modulo correctly. |
| Single-line mega-literal parser truncation | Extreme-rank literals parsed silently but dropped trailing declarations in earlier waves. | Mandatory multi-line W584 brace style for the 10-D inner literal. |
| Batch sweep blocked by unrelated giant specs | `./scripts/tri test --fast` Phase 1 Parse can stall on unrelated 4-MiBit specs. | Rely on direct Icarus/cocotb gates and document batch status. |

## Scientific / technical references

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array width as product of dimensions.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs as
  first-class synthesizable objects.
- Icarus Verilog Quirks / Extensions pages — packed-array subset behavior.
- Icarus issue #1134 — unpacked arrays of packed structs cause assertion
  failures; t27 flattening avoids the trigger.
- Icarus issue #1171 — large packed vectors can freeze elaboration; W598 stays
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
   - Adapt `/tmp/gen_w597.py` to `DIMS = [15] + [2] * 10`.
   - Use the W584 multi-line brace style.
   - Leaf value schedule: `x = (2*e + offset) % 32768`,
     `y = (2*e + offset + 1) % 32768`.
   - Add explicit modulo-wrap assertion with `make_grid(32768)[0]`.
   - Emit `pub const expected`, `pub var dst = make_grid(0)`, test and bench.

2. **Compiler / integration test**
   - Add `accepts_w598_bench_module_15x2p10_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.
   - Create empty Icarus baseline JSON.

3. **Seal and verify**
   - `cargo build --release -p t27c`.
   - `cargo test -p t27c --test icarus_lowerable`.
   - `t27c seal --save` on the witness.
   - Direct `t27c icarus-simulate` and `t27c icarus-cocotb`.

4. **Closeout**
   - `docs/reports/FPGA_LOOP_CLOSEOUT_W598_2026-07-07.md`.
   - Update `.trinity/experience.md`.
   - Persist memory `wave-loop-598.md` and `MEMORY.md` index update.
   - Commit with `Closes #1569`.

## Success criteria

- Witness parses, lowers, simulates, and passes cocotb reference model.
- `icarus_lowerable` test passes.
- No seal mismatches; FROZEN_HASH unchanged.
- Zero compiler or reference-model changes.
