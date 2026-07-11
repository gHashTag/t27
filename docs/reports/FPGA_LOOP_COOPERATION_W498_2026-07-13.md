# FPGA / Wave Loop Cooperation Variants — W498

**Date:** 2026-07-13
**From:** Wave Loop 497 close-out
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

W497 removed the architectural blocker for the generic structural equivalence
proof by totalizing the Icarus-lowerable predicates (`Predicate.lean`) and the
combinational evaluator (`SemanticsTotal.lean`). The generic theorem
`module_value_equiv_statement` is now stated with the right assumptions, but the
forward-simulation proof itself is not yet complete: one `sorry` remains in
`Soundness.lean`. The next wave should either finish that proof, shrink the
remaining emitter boundary, or collect live FPGA evidence. Three variants are
proposed below.

**Recommendation:** Select **Variant A (scoped)** to complete the
forward-simulation proof for the pure combinational subset first; relax the
reachability/closure assumptions and extend coverage to control flow only after
the core proof is closed. Select **Variant B** if the priority is to remove the
last documented Icarus baseline
(`w493_local_aos_element_field_not_lowerable.t27`). Select **Variant C** only if
the DLC10 cable and Wukong board are available for live cold-POR / SPI flash
evidence.

---

## Variant A — Complete the generic equivalence theorem (scoped)

**Goal:** Finish the forward-simulation proof of `module_value_equiv_statement`
for the pure combinational subset, then (time permitting) remove or weaken the
`Module.callsResolved` / `Module.callsReachable` assumptions and extend the total
evaluator to cover module-level `ifThenElse` / `forLoop` under a guarded
operational semantics.

**Why now:** W497 totalized every proof-relevant function, so the theorem is now
only blocked by the structural induction itself. Closing the core proof removes
the last `sorry` in the IcarusLowerable track and gives a reusable contract for
the combinational scalar subset before broadening it.

**Scope:**
- Prove the combined fuel/AST structural induction covering expressions
  (literals, identifiers, operators, `fieldAccess`, `index`, `call`,
  `structLit`/`arrayLit`), statements (`assign`, `varDecl`, `constDecl`,
  `return_ (some e)`, `bareCall`), statement lists, function inlining, module
  globals, and the named `main` function.
- Discharge the integer-literal string-roundtrip side condition
  (`String.toInt? (toString n) = some n`).
- Prove that `Module.isLowerable` together with a well-formed `Env.reachable`
  list implies `Module.callsResolved` and `Module.callsReachable` for the
  transitive call graph, or change `emitModule` to emit all functions and
  strengthen the soundness contract accordingly.
- Add a guarded big-step semantics for `ifThenElse` and `forLoop` in the total
  evaluator, restricted to statically known bounds, and include them in the
  generic theorem if time allows.
- Add adversarial witnesses that exercise conditionals in the lowerable subset.

**Deliverables:**
- `module_value_equiv_statement` proved with no remaining `sorry`.
- (Optional) Generic theorem with fewer or no reachability assumptions.
- (Optional) Guarded semantics for control flow in `SemanticsTotal.lean`.
- Regression: all W495/W497 witness theorems still pass.

**Risk:** Medium-to-high. The structural proof is bookkeeping-heavy; the string
roundtrip lemma and reachability closure add extra steps. Keeping the scope
scoped to the combinational proof first lowers risk.

---

## Variant B — Close the local AOS element boundary

**Goal:** Remove the last documented Icarus baseline:
`w493_local_aos_element_field_not_lowerable.t27`. Local non-memory-mode arrays
of structs are unpacked into per-element per-field registers, which prevents
packing an indexed element into a struct-literal concatenation.

**Why now:** The gate is green with exactly one documented Icarus baseline.
Removing it grows the lowerable corpus and gives the equivalence work more
concrete targets that exercise indexing and packing together.

**Scope:**
- Track local non-memory-mode arrays of structs so that a literal-index element
  can be packed via a priority mux or per-element register selection.
- Add positive witnesses for local AOS element used as a whole struct value.
- Add a new adversarial witness for the next boundary (e.g., variable-index
  local AOS element as a struct-literal field).
- Update `gen_verilog_iverilog_smoke_baseline.json` and reseal if the compiler
  changes.

**Deliverables:**
- 1–2 new positive scratch specs.
- 1 new adversarial scratch spec.
- Updated Icarus baseline JSON with zero documented failures.
- Green `./scripts/tri test --fast --icarus-lowerable` with zero disagreements.

**Risk:** Low to medium. The fix is localized to struct-literal/AOS lowering,
but may require careful tracking of local AOS metadata.

---

## Variant C — FPGA live cold-POR / SPI flash boot evidence

**Goal:** Collect live evidence that synthesized bitstreams boot correctly from
SPI flash across a sweep of OSCFSEL variants, using the in-repo `cli/dlc10`
driver and the QMTech Wukong V1 / XC7A100T-FGG676 board.

**Why now:** The formal track now has a generic equivalence theorem for the
combinational subset. A live run would ground that progress in hardware reality
and exercise the PVT worst-case bounds in `TernaryFPGABoot.lean`.

**Scope:**
- Flash a recent bitstream with `dlc10 flash`.
- Perform cold-POR boot and sweep OSCFSEL 0–7.
- Record `boot-log.json` / `smoke_gate_report.json`.
- Compare measured CCLK timing against the Lean PVT worst-case bounds.

**Deliverables:**
- `docs/reports/FPGA_W498_LIVE_EVIDENCE_2026-07-*.md`.
- Updated `fpga/HARDWARE_SSOT.md` if any deviations are found.
- Regression fixture if a reproducible failure is discovered.

**Risk:** High availability risk. Requires the DLC10 cable and Wukong board.
Defer if hardware is unavailable.

---

## Recommended default: Variant A (scoped)

W497 removed the proof-opacity blocker but left the generic theorem with one
`sorry`. The highest-leverage follow-through is to close that proof for the pure
combinational subset first, then harden the theorem by eliminating assumptions
and extending coverage to control flow. If Variant A uncovers an emitter
modeling gap for a construct, fold the corresponding Variant B fix into the
same wave and document any residual boundary.

---

*φ² + φ⁻² = 3 | TRINITY*
