# Wave Loop 470 — Cooperation Variants (2026-07-08)

**Issue:** #1448 (to create)  
**Source wave:** Wave Loop 469 (#1447)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 469 selects **Variant B** from the W469 cooperation plan: with the physical bench still blocked by the missing DLC10 cable / unwired P12 relay, the wave continues the `gen-verilog` compiler-backend hardening line. W469 targets four remaining struct/array gaps: multi-dimensional arrays of structs (`[M][N]Pt`), module-level scalar struct variables/constants, scalar struct parameters, and whole-struct comparison. The expected outcome is another green suite with zero IGLA failures.

Three candidate directions are offered for Wave Loop 470. The default recommendation is **Variant B** because the physical bench remains unavailable.

---

## Variant A — Live cold-POR CCLK sweep (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong XC7A100T bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T.
- Persist the captured fixtures under `tests/fixtures/fpga/theorem-matrix/live-w470/`.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g. `XADC_LIVE_W470_OPERATING_POINT`, closing the live-measurement → formal-claim loop.
- If W469 succeeds in struct lowering, add a struct-return benchmark in the live capture (e.g. read a struct of PVT sensors in one SPI transaction).

**Pros:** advances the physical boot-evidence line, which is the project's strongest differentiation.

**Cons:** blocked by hardware availability; cannot be the default.

---

## Variant B — Continue compiler-backend hardening (default if bench blocked)

**Trigger:** physical bench still unavailable (most likely).

**Work:**
1. **Arrays of structs returned from functions.** Extend struct-return packing so a function can return `[N]Pt` and callers can assign the result into a local/module array.
2. **Module-level arrays of structs with variable-index writes.** Currently module-level const struct arrays are read-only and emitted as ROMs; extend to `var mem : [N]Pt` with synthesizable per-field memory read/write and `(* ram_style = "..." *)` attribute support.
3. **Nested struct literal packing in expression contexts.** Allow `Pt{.x = Inner{...}}` where the inner struct is supplied as a whole rather than field-by-field, used in return values and comparisons.
4. **Struct fields of struct-array type.** Support `Outer { pts : [3]Pt }` at module level, in local variables, and as function parameters.

**Pros:** directly extends the struct-array line that W455–W469 have been hardening; no hardware dependency; maintains the zero-IGLA-failure streak.

**Cons:** does not produce new physical evidence.

**Recommended:** **Variant B** is the default for W470.

---

## Variant C — Formal fallback (if Variant B is blocked)

**Trigger:** module-level struct-array RAMs, function-returned struct arrays, or nested struct literals prove too large for one wave, or a regression-free implementation cannot be found quickly.

**Work:**
- Add a synthesizability theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean` for scalar struct parameter flattening (the Verilog field-wise inputs are equivalent to the source struct value).
- Add a whole-struct comparison correctness lemma that relates source `a == b` to the emitted field-wise conjunction.
- Add an adversarial witness that checks the emitted Verilog for any new `[M][N]Pt` scratch spec survives yosys elaboration without undeclared identifiers or width mismatches.

**Pros:** hardens the formal side of the compiler backend, giving Lean-native assurance even when the bench is blocked.

**Cons:** does not close user-facing compiler gaps as directly as Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before W470 planning is final, in which case switch to **Variant A**. If the module-level struct-array RAM / returned-struct-array refactor turns out to be larger than one wave, fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
