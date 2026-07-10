# Wave Loop 476 — Cooperation Variants (2026-07-07)

**Issue:** (to be opened)  
**Source wave:** Wave Loop 475 (parent)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 475 selected **Variant B** from the W475 cooperation plan: with the
physical FPGA bench still blocked by the missing DLC10 cable / unwired P12 relay,
the wave closed the nested-array-field equality and function-local array-of-struct
parameter-passing gaps. The conformance suite is green at **640/640** non-smoke
specs and **120/120** yosys smoke targets, with **zero** gen-verilog smoke failures.

Three candidate directions are offered for Wave Loop 476. The default
recommendation is **Variant B** because the bench remains unavailable and a small,
well-defined aggregate-lowering tail still exists.

---

## Variant A — Live cold-POR CCLK sweep (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong XC7A100T
bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T.
- Persist the captured fixtures under `tests/fixtures/fpga/theorem-matrix/live-w476/`.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g.
  `XADC_LIVE_W476_OPERATING_POINT`, closing the live-measurement → formal-claim
  loop.
- If possible, exercise the W475 aggregate lowering in a live benchmark (e.g.,
  read a structured PVT sensor bundle in one SPI transaction and pass it to a
  ternary MAC kernel).

**Pros:** advances the physical boot-evidence line, which is the project's
strongest differentiation.

**Cons:** blocked by hardware availability; cannot be the default.

---

## Variant B — Continue compiler-backend aggregate hardening (default if bench blocked)

**Trigger:** physical bench still unavailable (most likely).

**Work:**
1. **Local array-of-struct copy initializers.** Lower `var c : [2]Shape = b;`
   where `b` is another function-local array of structs with array-typed fields,
   by emitting per-field memory copies or a packed-vector round-trip.
2. **Module-level arrays of structs passed as packed-vector array parameters.**
   Unify the module-level array-parameter path with the local-packed path so a
   module array can also be passed as a packed vector when the callee expects it.
3. **Whole-struct assignment for nested structs with array-typed fields.** Lower
   `shape_a = shape_b` and `shapes_a = shapes_b` by value for scalar and array
   variables whose element struct contains array-typed fields.
4. **Adversarial yosys-elaboration witness for W476 scratch specs.** Add a
   regression that combines local-array copy initializers, module-array parameter
   passing, and whole-struct assignment of nested structs.
5. **Formal synthesizability lemmas (optional, if time permits).** Add Lean 4
   theorems in `proofs/lean4/Trinity/TernaryFPGABoot.lean` stating that the
   packed-vector memory model preserves source read/write semantics for arrays of
   structs with array-typed fields.

**Pros:** directly extends the struct-array line that W455–W475 have been
hardening; no hardware dependency; maintains the zero-IGLA-failure streak; the
remaining tail is small and reviewable.

**Cons:** does not produce new physical evidence.

**Recommended:** **Variant B** is the default for W476.

---

## Variant C — Formal fallback (if Variant B is blocked)

**Trigger:** the local-array copy / module-array parameter-passing tail proves
larger than one wave, or a regression-free implementation cannot be found
quickly.

**Work:**
- Add a synthesizability theorem for the packed-vector array-parameter model:
  passing a local array of structs as a packed vector and reading it back through
  the slice/mux path returns the source element values.
- Add a correctness lemma that array-of-struct equality for nested array fields
  compares the same bits as the source semantics.
- Add an adversarial yosys-elaboration witness for new `w476_*` scratch specs that
  checks for undeclared identifiers, width mismatches, and pragma placement.
- Refresh `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` to document the now-zero
  baseline and any residual master-side divergence.

**Pros:** hardens the formal side of the compiler backend, giving Lean-native
assurance even when the bench is blocked.

**Cons:** does not close the last user-facing compiler gaps as directly as
Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before
W476 planning is final, in which case switch to **Variant A**. If the local-array
copy / module-array parameter-passing tail turns out to be larger than one wave,
fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
