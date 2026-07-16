# Wave Loop 550 — 4-D primitive scalar array function returns for independent VCD cross-check

**Issue:** #1521 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-550`  
**Source:** `docs/reports/FPGA_LOOP_CLOSEOUT_W549_2026-07-16.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Extend Wave Loop 549's multi-dimensional primitive scalar array function-return
support to four-dimensional arrays (`[2][2][2][2]u8`).  This validates that the
row-major index linearization and bit-slice scaling are truly rank-independent
and reveals any remaining hard-coded 2-D or 3-D assumptions in the compiler,
reference model, or Lean formal model.

## Scope

1. Add a positive scratch witness `specs/scratch/w550_4d_call_init_returns_array.t27`
   with a function returning `[2][2][2][2]u8` and element extraction.
2. Confirm the existing `_collect_index_chain` / `_eval_index_bv` reference-model
   path handles four-level `ExprIndex` nesting.
3. Add lowerability and value-preservation theorems in
   `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` / `Soundness.lean`.
4. Record the Icarus baseline and seal the new witness.
5. If any hard-coded rank limit appears, remove it in the compiler, reference
   model, or Lean model.
6. Run the full validation matrix.

## Acceptance criteria

- The new 4-D primitive scalar array return witness passes Icarus simulation
  and the cocotb reference-model cross-check.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  0 cocotb failures, 0 Icarus failures, 0 seal mismatches, and the 24
  pre-existing yosys smoke baseline failures remain unchanged.
- `cargo build --release -p t27c`, `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
  `cargo test -p t27c --test icarus_lowerable`, and
  `lake build Trinity.IcarusLowerable.Soundness` remain green / 0 `sorry`.

---

*Next: Wave Loop 551 cooperation variants will be proposed in the W550 closeout.*
