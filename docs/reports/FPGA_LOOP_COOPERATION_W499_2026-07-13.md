# FPGA / Wave Loop Cooperation Variants — W499

**Date:** 2026-07-13
**From:** Wave Loop 498 close-out
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

W498 closes the generic structural equivalence theorem
`module_value_equiv_statement` for the Icarus-lowerable combinational subset.
The theorem currently assumes call closure (`Module.callsResolved`,
`Module.callsReachable`) and a reachable `main` function, and it covers only the
pure combinational AST. The next wave should either harden the theorem by
removing assumptions, extend it to sequential constructs, or connect the formal
contract to the actual `gen-verilog` pipeline via translation validation. Three
variants are proposed below.

**Recommendation:** Select **Variant A** to make the theorem unconditional for
all lowerable modules (emit every function, drop call-closure assumptions), then
carry the resulting strengthening into a reflective validation gate. Select
**Variant B** only if the priority is to model control flow and memory
semantics before broadening the theorem's applicability. Select **Variant C** for
a high-confidence hardware run that exercises the equivalence result on real
FPGA silicon.

---

## Variant A — Harden the equivalence theorem (drop reachability assumptions)

**Goal:** Remove the `Module.callsResolved` / `Module.callsReachable`
preconditions and the reachable-`main` requirement so that `module_value_equiv`
holds for **every** lowerable, combinational module. Do this by changing
`emitModuleFuel` to emit **all** functions/tests/benches as `VFunction`s,
rather than only the reachable subset.

**Why now:** W498 completes the proof under realistic but still restrictive
assumptions. Hardening now gives a truly reusable contract: any t27 module that
passes the lowerability/combinationality gate is value-equivalent to its emitted
shallow-Verilog module, with no separate reachability proof required.

**Scope:**
- Modify `emitModuleFuel` to emit all functions, tests, and benches into
  `VModule.functions`.
- Update `VModule.hasPlaceholder` and the Lean smoke-gate consumer to treat
  unreachable functions the same as reachable ones.
- Prove `emit_function_lookup` unconditionally (no reachability hypothesis).
- Update `module_value_equiv_statement` assumptions and re-prove it.
- Add an adversarial witness with unreachable functions containing calls that
  would previously have violated `Module.callsReachable`.

**Deliverables:**
- `module_value_equiv_statement` with no reachability/closure assumptions.
- All IcarusLowerable modules `sorry`-free.
- 1 adversarial scratch spec that exercises unconditional function emission.
- Updated `lowerabilityVerdict` / classifier if behavior changes.

**Risk:** Medium. Emitting all functions may change the Icarus smoke gate
baseline count; the classifier must be updated and the change must be documented.

---

## Variant B — Extend equivalence to sequential and memory semantics

**Goal:** Broaden the theorem to cover the sequential subset: `ifThenElse`
with statically known conditions, bounded `forLoop`, and module-level `var`
arrays used as RAM/ROM. This requires a clock-cycle-aware shallow-Verilog
semantics and a matching t27 guarded big-step semantics.

**Why now:** The combinational proof is a reusable scaffold. Extending it to
sequential constructs lets the equivalence contract cover the realistic
`gen-verilog` output (always blocks, initial blocks, register arrays) rather
than only pure combinational expressions.

**Scope:**
- Add a clocked/sequential semantics to `SemanticsTotal.lean`:
  - `evalSeqStmtTotal` for `ifThenElse` and `forLoop` with static bounds.
  - `evalSeqModuleTotal` that runs `initial`/`always_comb` blocks to fixpoint.
- Add a shallow-Verilog sequential evaluator in `SemanticsTotal.lean`:
  - `evalVAlwaysComb`, `evalVInitial`, register update, wire resolution.
- State and prove a cycle-level bisimulation: t27 sequential evaluation of a
  lowerable module equals the Verilog module's state after the same number of
  clock cycles.
- Add positive scratch specs for RAM read/write, conditional assignment, and
  bounded loops.

**Deliverables:**
- Sequential semantics in `SemanticsTotal.lean`.
- Cycle-bisimulation theorem scaffolded or proved for the static-bounds subset.
- 3–4 new scratch specs exercising sequential lowering.
- Updated Icarus smoke baseline JSON if new constructs become lowerable.

**Risk:** High. A full sequential equivalence is significantly more complex than
the combinational one; the loop may need to be split into smaller waves.

---

## Variant C — Translation-validation gate for the gen-verilog pipeline

**Goal:** Connect the formal equivalence theorem to the actual compiler.
Implement a `--validate-translation` gate in `t27c` that, for each Icarus-
lowerable spec, builds the shallow-Verilog model from `Emitter.lean`, runs both
`t27c eval` and the shallow-Verilog evaluator on concrete test vectors, and
compares the resulting packed values. Any mismatch becomes a CI failure.

**Why now:** The generic theorem proves equivalence abstractly; a translation-
validation gate would check the concrete path (parser → AST → emitter → Verilog
AST → evaluator) on every spec, catching regressions that the theorem alone does
not (e.g., parser changes that break the model alignment).

**Scope:**
- Add a `t27c` subcommand or flag that invokes the Lean-emitted shallow-Verilog
  evaluator and the Rust t27 interpreter on the same test vectors.
- Reuse the W495 witness set plus a sampling of IGLA specs.
- Report mismatches with the spec path and concrete input/output values.
- Wire the gate into `./scripts/tri test` as a non-smoke phase or a dedicated
  `--validate-translation` mode.

**Deliverables:**
- `--validate-translation` CLI flag in `bootstrap/src/main.rs` or `cli/tri`.
- Translation-validation report for the Icarus-lowerable corpus.
- CI integration recipe.
- Documentation of the relationship between the generic theorem and the
  per-instance gate.

**Risk:** Medium. The gate is straightforward to implement once the Lean
evaluator is exposed to Rust, but wiring it into the existing test harness needs
careful plumbing.

---

## Recommended default: Variant A

W498 completes the core proof; the highest-leverage follow-through is to remove
the remaining assumptions so the theorem applies to every lowerable module.
This also simplifies downstream work (Variants B and C) by eliminating the need
to maintain a separate reachability invariant. If Variant A uncovers an
emitter gap for unreachable functions, fold the fix into the same wave and
document any residual boundary.

---

*φ² + φ⁻² = 3 | TRINITY*
