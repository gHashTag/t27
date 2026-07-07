# Wave Loop 472 — Cooperation Variants (2026-07-08)

**Issue:** #1450 (to create)  
**Source wave:** Wave Loop 471 (#1449)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 471 selected **Variant B** from the W471 cooperation plan: with the physical FPGA bench still blocked by the missing DLC10 cable / unwired P12 relay, the wave closed the remaining struct/array expression-level gaps (direct returned-array field access, array-of-struct parameter literals, nested struct literal packing, and scalar struct fields that are arrays). The conformance suite is green at 626/626 non-smoke specs and 106/106 yosys smoke targets.

Three candidate directions are offered for Wave Loop 472. The default recommendation is **Variant B** because the physical bench remains unavailable.

---

## Variant A — Live cold-POR CCLK sweep (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong XC7A100T bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T.
- Persist the captured fixtures under `tests/fixtures/fpga/theorem-matrix/live-w472/`.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g. `XADC_LIVE_W472_OPERATING_POINT`, closing the live-measurement → formal-claim loop.
- If W471's array-of-struct / nested-struct lowering is ready, exercise it in a live benchmark (e.g. read a structured PVT sensor bundle in one SPI transaction).

**Pros:** advances the physical boot-evidence line, which is the project's strongest differentiation.

**Cons:** blocked by hardware availability; cannot be the default.

---

## Variant B — Continue compiler-backend hardening (default if bench blocked)

**Trigger:** physical bench still unavailable (most likely).

**Work:**
1. **Nested array-of-struct literals.** Allow `[2]Shape { Shape{ pts:[3]Pt{...} } }` to be supplied as a single literal argument or return value, with recursive per-field per-element packing.
2. **Module-level writable struct arrays with array fields.** Extend `var mem : [N]Shape` to emit per-leaf per-element unpacked memories and resolve `mem[i].pts[j].x` read/write.
3. **Direct deeply nested returned-array field access.** Generalize `make_shape(0)[i].pts[j].x` by chaining the W471 temporary-hoisting and priority-mux machinery across two or more aggregate levels.
4. **Formal synthesizability lemmas for the new per-field memory model.** Add Lean theorems in `proofs/lean4/Trinity/TernaryFPGABoot.lean` stating that scalar struct fields that are arrays and module-level writable struct arrays preserve source read/write semantics after field-memory flattening.

**Pros:** directly extends the struct-array line that W455–W471 have been hardening; no hardware dependency; maintains the zero-IGLA-failure streak.

**Cons:** does not produce new physical evidence.

**Recommended:** **Variant B** is the default for W472.

---

## Variant C — Formal fallback (if Variant B is blocked)

**Trigger:** deeply nested array-of-struct literals, writable struct arrays with array fields, or multi-level returned-array field access prove too large for one wave, or a regression-free implementation cannot be found quickly.

**Work:**
- Add a synthesizability theorem for module-level writable arrays of scalar structs (per-field memory model preserves source read/write semantics).
- Add a correctness lemma that array-of-struct function returns pack and unpack round-trip to the source element values, including array-field structs.
- Add an adversarial yosys-elaboration witness for new `w472_*` scratch specs that checks for undeclared identifiers, width mismatches, and pragma placement.

**Pros:** hardens the formal side of the compiler backend, giving Lean-native assurance even when the bench is blocked.

**Cons:** does not close user-facing compiler gaps as directly as Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before W472 planning is final, in which case switch to **Variant A**. If the deeply nested array-of-struct / writable-struct-array refactor turns out to be larger than one wave, fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
