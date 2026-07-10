# Wave Loop 477 — Cooperation Variants (2026-07-07)

**Issue:** (to be opened)  
**Source wave:** Wave Loop 476 (parent)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 476 selected **Variant B** from the W476 cooperation plan: with the
physical FPGA bench still blocked by the missing DLC10 cable / unwired P12 relay,
the wave closed the remaining aggregate-lowering tail (local AOS copy
initializers, module-array packed parameters, nested whole-struct assignment) by
adding four scratch specs and confirming the W475 backend infrastructure already
handles them. The conformance suite is green at **644/644** non-smoke specs and
**124/124** yosys smoke targets, with **zero** gen-verilog smoke failures.

Three candidate directions are offered for Wave Loop 477. The default
recommendation is **Variant B** because the bench remains unavailable and the
gen-verilog backend has a well-scoped hygiene issue that is now the most
valuable compiler-side follow-up.

---

## Variant A — Live cold-POR CCLK sweep (unblock if hardware available)

**Trigger:** DLC10 cable and P12/relay wiring are located and the Wukong XC7A100T
bench can be powered.

**Work:**
- Run a live cold-POR CCLK sweep on the Wukong XC7A100T.
- Persist the captured fixtures under `tests/fixtures/fpga/theorem-matrix/live-w477/`.
- Mint a new theorem in `proofs/lean4/Trinity/TernaryFPGABoot.lean`, e.g.
  `XADC_LIVE_W477_OPERATING_POINT`, closing the live-measurement → formal-claim
  loop.
- If possible, exercise the W475/W476 aggregate lowering in a live benchmark (e.g.,
  read a structured PVT sensor bundle in one SPI transaction, pass it to a
  ternary MAC kernel, and assert the result).

**Pros:** advances the physical boot-evidence line, which is the project's
strongest differentiation.

**Cons:** blocked by hardware availability; cannot be the default.

---

## Variant B — Function-body declaration hoisting for strict Verilog-2001 (default if bench blocked)

**Trigger:** physical bench still unavailable (most likely).

**Work:**
1. **Hoist all local declarations to the top of each generated Verilog function
   body.** Memory-mode local arrays (`reg [15:0] pts [0:1][0:2];`), scalar-struct
   temporaries, loop variables, and packed-vector temps currently appear after
   preceding assignments. Yosys tolerates this, but strict Verilog-2001 simulators
   such as Icarus reject it.
2. **Add Icarus Verilog simulation to the conformance gate.** Add a new suite
   phase that compiles each generated spec with `iverilog -g2012` and runs the
   emitted `initial begin assert(...)` test blocks with `vvp`.
3. **Keep Yosys smoke green.** Ensure the declaration-hoisting change does not
   break any existing yosys smoke targets or seals.
4. **Adversarial witness.** Add a scratch spec that deliberately interleaves local
   array declarations, scalar-struct temps, and loop variables to stress the
   hoisting logic.

**Pros:** makes the generated Verilog strict-standards compliant, opens the door
to broader simulator coverage, and is a natural next step after the aggregate
lowering is feature-complete. No hardware dependency.

**Cons:** does not produce new physical evidence; touches many generated files
and therefore requires a broad reseal.

**Recommended:** **Variant B** is the default for W477.

---

## Variant C — Formal fallback (if Variant B is blocked)

**Trigger:** declaration hoisting proves larger than one wave, or a
regression-free implementation cannot be found quickly.

**Work:**
- Add a synthesizability theorem for the packed-vector array-parameter model:
  passing a local or module array of structs as a packed vector and reading it
  back through the slice/mux path returns the source element values.
- Add a correctness lemma that whole-struct assignment and AOS equality for
  nested array fields compare/copy the same bits as the source semantics.
- Add an adversarial yosys-elaboration witness for new `w477_*` scratch specs that
  checks for undeclared identifiers, width mismatches, and pragma placement.
- Refresh `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` to document the now-zero
  baseline and any residual master-side divergence.

**Pros:** hardens the formal side of the compiler backend, giving Lean-native
assurance even when the bench is blocked.

**Cons:** does not close the last compiler-hygiene gap as directly as Variant B.

---

## Selection recommendation

Select **Variant B** unless the DLC10 cable / P12 relay become available before
W477 planning is final, in which case switch to **Variant A**. If declaration
hoisting turns out to be larger than one wave, fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
