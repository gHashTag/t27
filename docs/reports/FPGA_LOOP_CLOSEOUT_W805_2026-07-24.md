:# FPGA Loop Closeout — Wave Loop 805

**Date:** 2026-07-24
**Wave:** 805
**Issue:** #1539
**Branch:** `wave-loop-805`
**Base:** `wave-loop-804` @ `ab3782736bc2b270b4c3bb45cb78958a222e9a4e`
**PR:** #1540

## Variant under test

Module-scope non-power-of-two outer-dimension array-of-struct variable from a call with indexed signed writes:

```t27
[429][2][2][2][2][2][2] Pt
```

Struct definition:

```t27
pub struct Pt { x : i16, y : i16 }
```

## Scale

| Metric | Value |
|--------|-------|
| Outer dimension | 429 |
| Inner dimensions | 2^6 = 64 scalar cells per row |
| Total elements | 429 × 64 = 27,456 |
| Bits per element | 32 (2 × i16) |
| Packed vector width | 27,456 × 32 = **878,592 bits** |
| Size | ~0.838 MiBit |

Layout formula remains canonical:

```
element_index = r·64 + a5·32 + a4·16 + a3·8 + a2·4 + a1·2 + a0
bit_offset    = element_index · 32
vector_width  = 429 · 2048 = 878,592 bits
```

## What changed

- `scripts/gen_w805.py` — generator copied from `gen_w804.py` and updated to `OUTER = 429`, `MID_IDX = 214`. Generator copy hazard fixed before first run (destination path and module header f-string).
- `specs/scratch/w805_bench_module_429x2p6_aos_var_call_write.t27` — generated witness.
- `.trinity/seals/scratch_w805_bench_module_429x2p6_aos_var_call_write.json` — saved seal.
- `bootstrap/tests/icarus_lowerable.rs` — added `accepts_w805_bench_module_429x2p6_aos_var_call_write`.
- `docs/reports/FPGA_LOOP_CLOSEOUT_W805_2026-07-24.md` — this report.
- `.claude/plans/wave-loop-806.md` — next-wave plan.
- `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, `.claude/skills/t27-wave-loop.md` — coordination docs and live tracker updated.

## What did not change

- `bootstrap/stage0/FROZEN_HASH` remains unchanged:
  ```
  68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc bootstrap/src/compiler.rs
  ```
- No compiler source changes.
- No reference-model changes.

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo clippy -p t27c` | green (780 warnings, 0 errors) |
| `cargo test -p t27c --test icarus_lowerable` | **265/0** |
| `cargo test -p t27c --test bitnet_pipeline` | 20/0 |
| `cargo test -p t27c --test bitnet_top` | 17/0 |
| `t27c parse` | PASS |
| `t27c icarus-lowerable` | PASS |
| `t27c icarus-simulate` | PASS (17 cycles) |
| `t27c icarus-cocotb` | PASS |
| `t27c seal --save` | PASS |

## Generator copy hazard

The hazard struck again after copying `gen_w804.py` → `gen_w805.py`. Both the destination path and the module header f-string initially carried stale `w804`/`427` references. Fixed before first generation. Recommended remediation (already tracked in `.trinity/experience.md`): parameterize wave number and outer dimension from a single source-of-truth constant in the generator template.

## Weak-point audit (fresh scan)

No new actionable weak points introduced by this wave. The pre-existing `verilog_array_literal_expr` regression remains documented and is outside the scope of the mechanical witness ladder.

## 2024–2026 ternary / MVL literature scan

No new publications requiring design changes were identified during this closeout. The ladder continues to validate the same packed-vector lowering path that the BitNet-style ternary inference engine will reuse on the QMTech Wukong V1 / XC7A100T-FGG676 target.

## Cooperation variants for Wave Loop 806

- **A (recommended):** `[431][2]^6 Pt`, outer += 2, `MID_IDX = 215`. Continues the mechanical ladder with the lowest cognitive overhead.
- **B:** `[425][3]^6 Pt` — grow the second inner dimension to stress row-stride scaling while keeping the same outer dimension.
- **C:** `[425][2]^6 Pt` with negative-index writes to exercise wrap-around/underflow addressing in the packed vector.

## Significance for the ternary-FPGA target

Wave Loop 805 confirms that the t27c lowering path scales to 878,592-bit module-scope packed vectors with non-power-of-two outer dimension 429. This is the same path used by the BitNet/ternary inference engine: wide stateful arrays, deterministic addressing, and reference-model-correct simulation. Each successful rung reduces risk for the eventual full ternary model on the QMTech Wukong V1 FPGA via OpenXC7/Yosys/nextpnr.

## Next step

Wave Loop 806. Recommended variant A: `[431][2]^6 Pt`.

Phase complete: Verify
→ Phase 9: Learn
