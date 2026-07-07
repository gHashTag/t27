# Wave Loop 475 — Cooperation Variants (2026-07-07)

**Issue:** (to be opened)  
**Source wave:** Wave Loop 474 (parent #1447)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 474 selected **Variant B** from the W474 cooperation plan: with the physical FPGA bench still blocked by the missing DLC10 cable / unwired P12 relay, the wave closed the function-local nested-struct-array, array-of-struct return writeback, scalar-struct equality, and adversarial yosys-witness gaps. The conformance suite is green at **637/637** non-smoke specs and **117/117** yosys smoke targets, with **zero** gen-verilog smoke failures.

Three candidate directions are offered for Wave Loop 475. The default recommendation is **Variant B** because the bench remains unavailable and a small, well-defined aggregate-lowering tail still exists.

---

## Variant A — Live cold-POR CCLK sweep (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong XC7A100T bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T.
- Persist the captured fixtures under `tests/fixtures/fpga/theorem-matrix/live-w475/`.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g. `XADC_LIVE_W475_OPERATING_POINT`, closing the live-measurement → formal-claim loop.
- Exercise the W474 aggregate lowering in a live benchmark if possible (e.g., read a structured PVT sensor bundle in one SPI transaction).

**Pros:** advances the physical boot-evidence line, which is the project's strongest differentiation.

**Cons:** blocked by hardware availability; cannot be the default.

---

## Variant B — Continue compiler-backend aggregate hardening (default if bench blocked)

**Trigger:** physical bench still unavailable (most likely).

**Work:**
1. **Array-of-struct equality for nested array fields.** Extend the `==`/`!=` lowering so arrays of structs whose element struct itself has array-typed fields compare correctly, by teaching the packer to read multi-dimensional field memories.
2. **Whole-struct equality for nested structs with array-typed fields.** Lower equality for scalar structs that contain array-typed fields (e.g., `Shape { pts : [3]Pt }`) and for arrays of such structs.
3. **Function-local arrays of structs passed as array parameters.** Allow a local array of structs to be passed to a function with a matching array parameter, including the memory-mode layout for array-typed fields.
4. **Adversarial yosys-elaboration witness for new W475 scratch specs.** Add a regression that combines nested-array-field equality, local AOS parameter passing, and module-level AOS return writeback.
5. **Formal synthesizability lemmas (optional, if time permits).** Add Lean 4 theorems in `proofs/lean4/Trinity/TernaryFPGABoot.lean` stating that the per-field memory model preserves source read/write semantics for arrays of structs with array-typed fields.

**Pros:** directly extends the struct-array line that W455–W474 have been hardening; no hardware dependency; maintains the zero-IGLA-failure streak; the remaining tail is small and reviewable.

**Cons:** does not produce new physical evidence.

**Recommended:** **Variant B** is the default for W475.

---

## Variant C — Formal fallback (if Variant B is blocked)

**Trigger:** the nested-field equality / local AOS parameter-passing tail proves larger than one wave, or a regression-free implementation cannot be found quickly.

**Work:**
- Add a synthesizability theorem for module-level and function-local arrays of structs with array-typed fields (per-field memory model preserves source read/write semantics).
- Add a correctness lemma that array-of-struct function returns and module/local initializers pack and unpack round-trip to the source element values.
- Add an adversarial yosys-elaboration witness for new `w475_*` scratch specs that checks for undeclared identifiers, width mismatches, and pragma placement.
- Refresh `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` to document the now-zero baseline and any residual master-side divergence.

**Pros:** hardens the formal side of the compiler backend, giving Lean-native assurance even when the bench is blocked.

**Cons:** does not close the last user-facing compiler gaps as directly as Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before W475 planning is final, in which case switch to **Variant A**. If the nested-field equality / local AOS parameter-passing tail turns out to be larger than one wave, fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
