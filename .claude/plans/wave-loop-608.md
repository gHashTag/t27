# Wave Loop 608 Plan — `[35][2]^6 Pt` module-scope AoS var from call

**Issue:** #1579  
**Branch:** `wave-loop-608`  
**Previous:** Wave Loop 607 (#1578, `wave-loop-607`)  
**Date:** 2026-07-07  
**Estimated complexity:** Low — zero compiler/reference-model changes expected.

## Goal

Validate a module-scope, mutable, packed array-of-scalar-struct variable with a
non-power-of-two outer dimension of **35**, initialized from a function call, and
exercised with indexed signed field writes and read-back.

## Variant rationale

- **Variant A (chosen):** `[35][2]^6 Pt` — continues the odd outer-dimension
  ladder (3 → 5 → 7 → 9 → 11 → 13 → 15 → 17 → 19 → 21 → 23 → 25 → 27 → 29 → 31 → 33 → 35). The 0.068 MiBit
  total is small enough for very fast direct simulation while still proving the
  compiler's generic packed-AoS paths handle outer stride 35 end-to-end.
- **Variant B (rejected):** `[2]^18 Pt` — crosses the 4-MiBit cliff; risks
  Icarus/Yosys capacity or runaway parse time without chunked-literal support.
- **Variant C (deferred):** `[35][2]^6 Pt` with conditional whole-array
  reassignment — useful, but the priority is extending the non-p2 outer ladder.

## Sizing

For `[35][2]^6 Pt`:

- Elements = `35 × 2^6 = 2,240`.
- Bits = `2,240 × 32 = 71,680`.
- Expected witness file ≈ 0.28 MB (multi-line brace style).

## Weak-point analysis

| Weak point | Why it matters | Mitigation in this wave |
|------------|----------------|---------------------------|
| Outer dimension 35 untested at module scope | Stride-35 multiplication and row-major flattening must be correct in compiler, simulator, and reference model. | Direct Icarus simulation + cocotb reference-model cross-check. |
| Element count below the modulo-wrap point | With 2,240 elements the offset-0 schedule `(2*e + offset) % 32768` never exceeds `i16` range (`max raw = 4,479`). Earlier large waves used `e_wrap = 16384` to assert wrap behavior. | Add an explicit shifted call `make_grid(32768)` and assert its first/last elements equal `(offset + raw) % 32768`, proving the reference model and compiler both evaluate modulo correctly. |
| Single-line mega-literal parser truncation | Extreme-rank literals parsed silently but dropped trailing declarations in earlier waves. | Mandatory multi-line W584 brace style for the 6-D inner literal. |
| Full batch sweep blocked by unrelated giant specs | `./scripts/tri test --fast` Phase 1 Parse can stall on unrelated 4-MiBit specs. | Rely on direct Icarus/cocotb gates and document batch status. |
| Syntax compatibility with the lowerable subset | W606 showed that bench-local `mut dst` and compact struct literals can parse but produce invalid Verilog. | Reuse the exact W605/W606/W607 module-scope lowerable style: `pub var dst`, `pub const expected`, explicit array-type annotations, `.x = ...` field initializers, separate `test`/`bench` blocks. |

## Scientific / technical references

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array width as product of dimensions.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs as
  first-class synthesizable objects.
- Icarus Verilog Quirks / Extensions pages — packed-array subset behavior.
- Icarus issue #1134 — unpacked arrays of packed structs cause assertion
  failures; t27 flattening avoids the trigger.
- Icarus issue #1171 — large packed vectors can freeze elaboration; W608 stays
  far below the reported threshold.
- Yosys docs / PR #4100 / issue #4653 / issue #2677 — multidimensional packed
  arrays supported, arrays of packed structs unsupported; t27 flattening
  avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals and flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

## Decomposition

1. **Spec generation**
   - Adapt `/tmp/gen_w607.py` to `DIMS = [35] + [2] * 6`.
   - Use the W584 multi-line brace style.
   - Leaf value schedule: `x = (2*e + offset) % 32768`,
     `y = (2*e + offset + 1) % 32768`.
   - Add explicit modulo-wrap assertion with `make_grid(32768)`.
   - Emit `pub const expected`, `pub var dst = make_grid(0)`, test and bench.

2. **Compiler / integration test**
   - Add `accepts_w608_bench_module_35x2p6_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.
   - Create empty Icarus baseline JSON.

3. **Seal and verify**
   - `cargo build --release -p t27c`.
   - `cargo test -p t27c --test icarus_lowerable`.
   - `t27c seal --save` on the witness.
   - Direct `t27c icarus-simulate` and `t27c icarus-cocotb`.

4. **Closeout**
   - `docs/reports/FPGA_LOOP_CLOSEOUT_W608_2026-07-07.md`.
   - Update `.trinity/experience.md`.
   - Persist memory `wave-loop-608.md` and `MEMORY.md` index update.
   - Commit with `Closes #1579`.

## Success criteria

- Witness parses, lowers, simulates, and passes cocotb reference model.
- `icarus_lowerable` test passes.
- No seal mismatches; FROZEN_HASH unchanged.
- Zero compiler or reference-model changes.
