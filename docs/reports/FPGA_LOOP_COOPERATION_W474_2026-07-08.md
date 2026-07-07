# Wave Loop 474 — Cooperation Variants (2026-07-08)

**Issue:** (to be opened)  
**Source wave:** Wave Loop 473 (#1447)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 473 selected **Variant B** from the W473 cooperation plan: with the physical FPGA bench still blocked by the missing DLC10 cable / unwired P12 relay, the wave closed the multi-dimensional outer-array linearization gap for module-level arrays of structs with array-typed fields. The conformance suite is green at **633/633** non-smoke specs and **113/113** yosys smoke targets, with **zero** gen-verilog smoke failures.

Three candidate directions are offered for Wave Loop 474. The default recommendation is **Variant B** because the bench remains unavailable and a small, well-defined aggregate-lowering tail still exists.

---

## Variant A — Live cold-POR CCLK sweep (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong XC7A100T bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T.
- Persist the captured fixtures under `tests/fixtures/fpga/theorem-matrix/live-w474/`.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g. `XADC_LIVE_W474_OPERATING_POINT`, closing the live-measurement → formal-claim loop.
- Exercise the W473 aggregate lowering in a live benchmark if possible (e.g., read a structured PVT sensor bundle in one SPI transaction).

**Pros:** advances the physical boot-evidence line, which is the project's strongest differentiation.

**Cons:** blocked by hardware availability; cannot be the default.

---

## Variant B — Continue compiler-backend aggregate hardening (default if bench blocked)

**Trigger:** physical bench still unavailable (most likely).

**Work:**
1. **Function-local arrays of structs with array-typed fields.** Extend the per-leaf per-element register model to local declarations such as `var tmp : [2][3]Shape` so that `tmp[i].pts[j].x = v` and read-back work inside functions and bench blocks.
2. **Array-of-struct function returns with nested field writeback.** Allow a function returning `[N]Shape` to be assigned to a module-level variable or used as a module-level `const` initializer, with per-leaf memory population verified by yosys.
3. **Whole-struct comparison and scalar-struct equality.** Lower `==` and `!=` for scalar structs and small arrays of structs into packed bitwise comparisons.
4. **Adversarial yosys-elaboration witness for new W474 scratch specs.** Add a regression that checks generated Verilog for undeclared identifiers, width mismatches, and illegal inline declarations before simulation.
5. **Formal synthesizability lemmas (optional, if time permits).** Add Lean 4 theorems in `proofs/lean4/Trinity/TernaryFPGABoot.lean` stating that the per-field memory model preserves source read/write semantics for arrays of structs with array-typed fields.

**Pros:** directly extends the struct-array line that W455–W473 have been hardening; no hardware dependency; maintains the zero-IGLA-failure streak; the remaining tail is small and reviewable.

**Cons:** does not produce new physical evidence.

**Recommended:** **Variant B** is the default for W474.

---

## Variant C — Formal fallback (if Variant B is blocked)

**Trigger:** the function-local / return-with-writeback / equality tail proves larger than one wave, or a regression-free implementation cannot be found quickly.

**Work:**
- Add a synthesizability theorem for module-level arrays of structs with array-typed fields (per-field memory model preserves source read/write semantics).
- Add a correctness lemma that array-of-struct function returns and module-level initializers pack and unpack round-trip to the source element values.
- Add an adversarial yosys-elaboration witness for new `w474_*` scratch specs that checks for undeclared identifiers, width mismatches, and pragma placement.
- Refresh `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` to document the now-zero baseline and any residual master-side divergence.

**Pros:** hardens the formal side of the compiler backend, giving Lean-native assurance even when the bench is blocked.

**Cons:** does not close the last user-facing compiler gaps as directly as Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before W474 planning is final, in which case switch to **Variant A**. If the function-local / return-with-writeback / equality tail turns out to be larger than one wave, fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
