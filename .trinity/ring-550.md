# Ring 550 — 4-D primitive scalar array function returns

**Date:** 2026-07-16  
**Issue:** #1521  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Core insight

Wave Loops 545–549 increased rank one step at a time.  Wave Loop 550 asked:
"is the code actually rank-independent, or does it only happen to work for ranks
1–3?"  The answer was that the code is rank-independent: no compiler,
reference-model, or Lean changes were needed for rank 4.

Key evidence:
- Generated Verilog for `[2][2][2][2]u8`: `m[(((a * 8) + (b * 4) + (c * 2) + d) * 8) +: 8]`.
- Python reference model: `_collect_index_chain` walks four `ExprIndex` nodes and
  `_eval_index_bv` computes the row-major flat index.
- Lean model: nested `.array` depth 4 and repeated `.index` evaluate with the
  same layout.

## Decision record

- Chose a compact `[2][2][2][2]u8` shape so the total packed width (128 bits)
  stays within the same VCD probe slice regime as previous witnesses.
- Did not add a signed 4-D witness; signed element semantics are rank-
  independent and already covered by W548's signed 2-D witness.
- Promoted the next loop from rank 5 to deterministic `bench`-block cross-check,
  because rank-independence has been sufficiently demonstrated.

## Verification shortcuts

- `./target/release/t27c icarus-cocotb <spec>` is the fastest end-to-end check.
- `cargo test -p t27c --test icarus_lowerable` catches classifier regressions.
- `lake build Trinity.IcarusLowerable.Soundness` confirms formal model agreement.

## Next ring

Wave Loop 551 (Variant A): extend the cocotb reference-model cross-check to
deterministic `bench` blocks, broadening verification from `test` to cycle-level
performance assertions.
