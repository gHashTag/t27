# Wave Loop 597 Plan — `[13][2]^11 Pt` module-scope AoS var from call

**Issue:** #1568  
**Branch:** `wave-loop-597`  
**Previous:** Wave Loop 596 (#1567, `wave-loop-596`)  
**Date:** 2026-07-07  
**Estimated complexity:** Low — zero compiler/reference-model changes expected.

## Goal

Validate a module-scope, mutable, packed array-of-scalar-struct variable with a
non-power-of-two outer dimension (13), initialized from a function call, and
exercised with indexed signed field writes and read-back.

## Variant rationale

- **Variant A (chosen):** `[13][2]^11 Pt` — continues the odd outer-dimension
  ladder (3 → 5 → 7 → 9 → 11 → 13). The 0.81 MiBit total is small enough for
  fast direct simulation while still proving the compiler's generic packed-AoS
  paths handle outer stride 13 end-to-end.
- **Variant B (rejected):** `[2]^18 Pt` — crosses the 4-MiBit cliff; risks
  Icarus/Yosys capacity or runaway parse time without chunked-literal support.
- **Variant C (deferred):** `[13][2]^11 Pt` with conditional whole-array
  reassignment — useful, but the priority is extending the non-p2 outer ladder.

## Corrected sizing

The W596 closeout report mistakenly listed this variant as 1,114,112 bits /
34,816 elements. For `[13][2]^11 Pt`:

- Elements = `13 × 2^11 = 26,624`.
- Bits = `26,624 × 32 = 852,032`.
- Expected witness file ≈ 5–6 MB (multi-line brace style).

## Decomposition

1. **Spec generation**
   - Adapt `/tmp/gen_w596.py` to `DIMS = [13] + [2] * 11`.
   - Use the W584 multi-line brace style.
   - Leaf value schedule: `x = (2*e + offset) % 32768`,
     `y = (2*e + offset + 1) % 32768`.
   - Emit `pub const expected`, `pub var dst = make_grid(0)`, test and bench.

2. **Compiler / integration test**
   - Add `accepts_w597_bench_module_13x2p11_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs`.
   - Create empty Icarus baseline JSON.

3. **Seal and verify**
   - `cargo build --release -p t27c`.
   - `cargo test -p t27c --test icarus_lowerable`.
   - `t27c seal --save` on the witness.
   - Direct `t27c icarus-simulate` and `t27c icarus-cocotb`.
   - Fast repo sweep `./scripts/tri test --fast`.

4. **Closeout**
   - `docs/reports/FPGA_LOOP_CLOSEOUT_W597_2026-07-07.md`.
   - Update `.trinity/experience.md`.
   - Persist memory `wave-loop-597.md` and `MEMORY.md` index.
   - Commit with `Closes #1568`.

## Risk mitigations

| Risk | Mitigation |
|------|------------|
| Outer stride 13 untested at module scope | Add direct simulation + cocotb reference-model cross-check. |
| Signed i16 overflow | `% 32768` schedule; max raw value 53247. |
| Single-line mega-literal parser truncation | Mandatory multi-line W584 brace style. |
| Full batch sweep too slow on large literal | Rely on direct Icarus/cocotb gates and fast sweep; document batch status. |

## Success criteria

- Witness parses, lowers, simulates, and passes cocotb reference model.
- `icarus_lowerable` test passes.
- No seal mismatches; FROZEN_HASH unchanged.
- Zero compiler or reference-model changes.
