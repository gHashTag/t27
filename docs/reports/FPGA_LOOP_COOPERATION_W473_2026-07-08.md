# Wave Loop 473 — Cooperation Variants (2026-07-08)

**Issue:** #1447  
**Source wave:** Wave Loop 472 (#1448)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 472 selected **Variant B** from the W472 cooperation plan: with the physical FPGA bench still blocked by the missing DLC10 cable / unwired P12 relay, the wave closed the deepest remaining `gen-verilog` aggregate-lowering gaps. The conformance suite is green at **629/629** non-smoke specs and **109/109** yosys smoke targets, with **zero** gen-verilog smoke failures for the first time in this hardening line.

Three candidate directions are offered for Wave Loop 473. The default recommendation is **Variant B** because the physical bench remains unavailable and the aggregate-lowering line still has a small, well-defined tail.

---

## Variant A — Live cold-POR CCLK sweep (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong XC7A100T bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T.
- Persist the captured fixtures under `tests/fixtures/fpga/theorem-matrix/live-w473/`.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g. `XADC_LIVE_W473_OPERATING_POINT`, closing the live-measurement → formal-claim loop.
- Exercise the W472 aggregate lowering in a live benchmark if possible (e.g., read a structured PVT sensor bundle in one SPI transaction).

**Pros:** advances the physical boot-evidence line, which is the project's strongest differentiation.

**Cons:** blocked by hardware availability; cannot be the default.

---

## Variant B — Continue compiler-backend hardening (default if bench blocked)

**Trigger:** physical bench still unavailable (most likely).

**Work:**
1. **Writable nested struct-array field assignment.** Add explicit write-and-read-back coverage for `shapes[i].pts[j].x = v`, ensuring the per-leaf per-element register model updates correctly.
2. **Higher-dimensional struct arrays.** Extend the per-field per-element lowering to 3-D and mixed-dimension arrays of structs (`[2][3]Shape`, `[4][2][3]Pt`).
3. **Array-of-struct module-level initializers with deep nesting.** Stress-test `[2]Shape{ Shape{ pts:[3]Pt{...} } }` as a module-level `var` initializer through the yosys smoke gate.
4. **Adversarial yosys-elaboration witness for new W473 scratch specs.** Add a regression that checks generated Verilog for undeclared identifiers, width mismatches, and illegal inline declarations before simulation.
5. **Formal synthesizability lemmas (optional, if time permits).** Add Lean 4 theorems in `proofs/lean4/Trinity/TernaryFPGABoot.lean` stating that the per-field memory model preserves source read/write semantics for module-level arrays of structs with array-typed fields.

**Pros:** directly extends the struct-array line that W455–W472 have been hardening; no hardware dependency; maintains the zero-IGLA-failure streak; the remaining tail is small and reviewable.

**Cons:** does not produce new physical evidence.

**Recommended:** **Variant B** is the default for W473.

---

## Variant C — Formal fallback (if Variant B is blocked)

**Trigger:** the writable-assignment / higher-dimensional / initializer tail proves larger than one wave, or a regression-free implementation cannot be found quickly.

**Work:**
- Add a synthesizability theorem for module-level arrays of structs with array-typed fields (per-field memory model preserves source read/write semantics).
- Add a correctness lemma that array-of-struct function returns and module-level initializers pack and unpack round-trip to the source element values.
- Add an adversarial yosys-elaboration witness for new `w473_*` scratch specs that checks for undeclared identifiers, width mismatches, and pragma placement.
- Refresh `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` to document the now-zero baseline and any residual master-side divergence.

**Pros:** hardens the formal side of the compiler backend, giving Lean-native assurance even when the bench is blocked.

**Cons:** does not close the last user-facing compiler gaps as directly as Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before W473 planning is final, in which case switch to **Variant A**. If the writable-assignment / higher-dimensional tail turns out to be larger than one wave, fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
