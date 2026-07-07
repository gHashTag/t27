# Wave Loop 471 — Cooperation Variants (2026-07-08)

**Issue:** #1449 (to create)  
**Source wave:** Wave Loop 470 (#1448)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 470 selected **Variant B** from the W470 cooperation plan: with the physical bench still blocked by the missing DLC10 cable / unwired P12 relay, the wave closed the remaining struct/array lowering gaps (2-D scalar array parameter literals, arrays of structs returned from functions, and module-level writable arrays of structs). The conformance suite is green at 622/622 non-smoke specs and 102/102 yosys smoke targets.

Three candidate directions are offered for Wave Loop 471. The default recommendation is **Variant B** because the physical bench remains unavailable.

---

## Variant A — Live cold-POR CCLK sweep (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong XC7A100T bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T.
- Persist the captured fixtures under `tests/fixtures/fpga/theorem-matrix/live-w471/`.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g. `XADC_LIVE_W471_OPERATING_POINT`, closing the live-measurement → formal-claim loop.
- If W470's array-of-struct return lowering is ready, exercise it in a live benchmark (e.g. read a struct of PVT sensors in one SPI transaction).

**Pros:** advances the physical boot-evidence line, which is the project's strongest differentiation.

**Cons:** blocked by hardware availability; cannot be the default.

---

## Variant B — Continue compiler-backend hardening (default if bench blocked)

**Trigger:** physical bench still unavailable (most likely).

**Work:**
1. **Nested struct literal packing in expression contexts.** Allow `Pt{.x = Inner{...}}` where the inner struct is supplied as a whole rather than field-by-field, used in return values, comparisons, and array assignments.
2. **Struct fields that are arrays.** Support `struct Pt { coords : [3]u8 }` at module level, in local variables, and as function parameters, including read/write to `p.coords[i]`.
3. **Direct field access on returned arrays of structs.** Generalize `make_pts(0)[0].x` by hoisting the packed return vector into a temporary and emitting per-field index access.
4. **Array-of-struct parameter literal arguments.** Extend the anonymous-ROM binding pass so that function parameters of type `[N]Pt` can accept literal array-of-struct arguments deterministically.

**Pros:** directly extends the struct-array line that W455–W470 have been hardening; no hardware dependency; maintains the zero-IGLA-failure streak.

**Cons:** does not produce new physical evidence.

**Recommended:** **Variant B** is the default for W471.

---

## Variant C — Formal fallback (if Variant B is blocked)

**Trigger:** nested struct literal packing, struct fields that are arrays, or returned-array field access prove too large for one wave, or a regression-free implementation cannot be found quickly.

**Work:**
- Add a synthesizability theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean` for module-level writable struct arrays (the per-field memory model preserves source read/write semantics).
- Add a correctness lemma that array-of-struct function returns pack and unpack round-trip to the source element values.
- Add an adversarial yosys-elaboration witness for new `w471_*` scratch specs that checks for undeclared identifiers, width mismatches, and pragma placement.

**Pros:** hardens the formal side of the compiler backend, giving Lean-native assurance even when the bench is blocked.

**Cons:** does not close user-facing compiler gaps as directly as Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before W471 planning is final, in which case switch to **Variant A**. If the nested struct literal / struct-field-array refactor turns out to be larger than one wave, fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
