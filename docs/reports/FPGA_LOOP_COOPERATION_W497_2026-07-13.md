# FPGA / Wave Loop Cooperation Variants — W497

**Date:** 2026-07-13
**From:** Wave Loop 496 close-out
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

W496 mapped the remaining proof path for the generic structural equivalence
theorem. The immediate blocker is that the t27 and shallow-Verilog evaluators in
`Semantics.lean` are `partial` mutual definitions and therefore opaque to proofs.
The next wave should either totalize the semantics and finish the theorem,
shrink the remaining emitter boundary, or collect live FPGA evidence. Three
variants are proposed below.

**Recommendation:** Select **Variant A** to introduce a fuel-based total
semantics for the Icarus-lowerable combinational subset, prove the generic
structural equivalence theorem on that total evaluator, and bridge it back to the
existing partial evaluator with `native_decide` on the W495 witness set. Select
**Variant B** if the priority is to grow the lowerable corpus by closing the
local AOS element boundary before investing in proof automation. Select
**Variant C** only if the DLC10 cable and Wukong board are available for live
cold-POR / SPI flash evidence.

---

## Variant A — Totalize semantics and prove the generic equivalence theorem

**Goal:** Remove the `sorry` in `module_value_equiv_statement` by first
rewriting the combinational evaluator with an explicit `fuel : Nat` parameter,
then proving the generic theorem by structural induction over the lowerable AST,
and finally showing that the fuel-based evaluator agrees with the existing
`partial` evaluator on every concrete witness.

**Why now:** W496 isolated the exact blocker. The custom `Expr` recursor, the
valuation invariant, and the combinational subset predicate are all in place.
A fuel-based total evaluator is the smallest architectural change that makes the
evaluators proof-transparent while preserving their computational behavior for
the witness set.

**Scope:**
- Rewrite `evalExpr`, `evalVExpr`, `evalStmts`, and `evalVStmts` as total
  functions taking a `fuel : Nat` parameter and returning `Option` on fuel
  exhaustion.
- Prove that fuel-based evaluation is deterministic and that the fuel can be
  chosen large enough for every lowerable module (a structural size bound is
  sufficient).
- Prove expression equivalence by structural induction using
  `AstInduction.lean`.
- Prove statement-list equivalence for `assign`, `varDecl`, `constDecl`, and
  `return_` under `Valuation.equiv`.
- Lift to `evalFunction` / `evalVFunction`, module globals, and `main`.
- Add `native_decide` bridge lemmas showing the old and new evaluators agree on
  the W495 witnesses.

**Deliverables:**
- `module_value_equiv_statement` proved with zero `sorry`.
- A fuel-total `SemanticsTotal.lean` module.
- Bridge lemmas connecting total and partial evaluators on concrete modules.
- Regression: all W494/W495 witness theorems still pass.

**Risk:** Medium. The totalization is mechanical but touches every evaluator
function. The structural proof over nested inductives is nontrivial but the
scaffolding from W496 reduces it to a well-understood induction.

---

## Variant B — Close the local AOS element boundary

**Goal:** Remove the last documented Icarus baseline:
`w493_local_aos_element_field_not_lowerable.t27`. Local non-memory-mode arrays
of structs are unpacked into per-element per-field registers, which prevents
packing an indexed element into a struct-literal concatenation.

**Why now:** The gate is green with exactly one documented Icarus baseline.
Removing it grows the lowerable corpus, gives the equivalence work more concrete
targets that exercise indexing and packing together, and may uncover useful
invariants before the generic proof is attempted.

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

**Why now:** The formal track now has a clear path to the generic theorem. A
live run would ground that progress in hardware reality and exercise the PVT
worst-case bounds in `TernaryFPGABoot.lean`.

**Scope:**
- Flash a recent bitstream with `dlc10 flash`.
- Perform cold-POR boot and sweep OSCFSEL 0–7.
- Record `boot-log.json` / `smoke_gate_report.json`.
- Compare measured CCLK timing against the Lean PVT worst-case bounds.

**Deliverables:**
- `docs/reports/FPGA_W497_LIVE_EVIDENCE_2026-07-*.md`.
- Updated `fpga/HARDWARE_SSOT.md` if any deviations are found.
- Regression fixture if a reproducible failure is discovered.

**Risk:** High availability risk. Requires the DLC10 cable and Wukong board.
Defer if hardware is unavailable.

---

## Recommended default: Variant A

W496 produced all the scaffolding the generic theorem needs. The highest-leverage
follow-through is to totalize the evaluator and finish the proof. If the
totalization uncovers an emitter modeling gap for a construct, fold the
corresponding Variant B fix into the same wave and document any residual
boundary.

---

*φ² + φ⁻² = 3 | TRINITY*
