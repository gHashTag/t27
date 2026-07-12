# Wave Loop 504 — Decomposed Plan

**Issue:** #1473 (placeholder — GH_TOKEN unavailable)  
**Branch:** `wave-loop-504`  
**Variant:** A — extend generic Icarus equivalence theorem to bounded `forLoop`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Literature / weak-point review

### Weak points closed by W503
- `ifThenElse` added to the t27 and shallow-Verilog operational semantics.
- `ifThenElse` covered by the generic `module_value_equiv_statement` theorem.
- Bounded `forLoop` added to the model and the emitter.

### Remaining weak points
- **Generic theorem still rejects `forLoop`:** W503 left bounded loops as
  "lowerable but non-combinational".  Real scalar accumulator / summation loops
  are therefore outside the generic equivalence contract.
- **Fuel semantics for loops not aligned with induction:** the total loop
  evaluators currently evaluate the body at `fuel + 1`, which prevents a
  straightforward `all_equiv` induction case.
- **No adversarial sequential coverage:** only two tiny sequential witnesses
  exist; mixed `if`/`for` patterns are untested.

### Scientific / engineering anchors
- **CompCert Clight / Cminor `for` semantics** — bounded loops as repeated
  statement execution; forward-simulation proof uses a loop invariant.
  ([Leroy et al., *CompCert*](https://compcert.org/))
- **CompCert `Csharpminor` / `RTLgenproof`** — structural induction over
  control-flow with a nested iteration induction for `for` loops.
- **Icarus Verilog LRM** — `for` loops in procedural contexts are supported and
  the emitter already unrolls / emits them.

---

## 2. Task decomposition

### Phase 1 — Sequential predicate (1 subtask)
1.1 Add `Stmt.isSequentialFuel` / `Stmt.isSequentialListFuel` and structural
    `Stmt.isSequential'` / `Stmt.isSequentialList'` in `Predicate.lean`.
    - Same rules as combinational, but `forLoop` is allowed when its range and
      body are sequential.
1.2 Add `Function.isSequential` and `Module.isSequential`.
1.3 Prove `Module.isCombinational env m → Module.isSequential env m`.

### Phase 2 — Loop fuel alignment (1 subtask)
2.1 Change `evalForLoopTotal` and `evalVForLoopTotal` so each loop iteration
    consumes one unit of fuel (body evaluated at the smaller fuel).  This lets
    the `all_equiv` induction hypothesis cover the loop body.
2.2 Update zero-fuel lemmas in `Equivalence.lean` if needed.

### Phase 3 — Equivalence proof (2 subtasks)
3.1 Generalize `P_expr` / `P_stmt` / `P_stmts` / `P_function` in
    `Equivalence.lean` to use `Stmt.isSequential` instead of `Stmt.isCombinational`.
3.2 Add the `forLoop` case to `all_equiv` using a nested induction on the
    iteration count and the sequential-body induction hypothesis.
3.3 Add `module_value_equiv_proved_sequential` /
    `module_value_equiv_statement_sequential`.

### Phase 4 — Witness (1 subtask)
4.1 Write `specs/scratch/w504_for_sum.t27` — bounded `for` that returns a
    computed value, e.g. `1 + 2 + 3` or a running sum with parameter.
4.2 Add the witness env/module to `Lemmas.lean`.
4.3 Prove `w504_for_sum_value_equiv` via
    `module_value_equiv_statement_sequential` in `Soundness.lean`.
4.4 Save the seal.

### Phase 5 — Verification (1 subtask)
5.1 `lake build Trinity.IcarusLowerable.Soundness` green, zero `sorry`.
5.2 `./scripts/tri verify --lean-lowerable` zero disagreements.
5.3 `./scripts/tri test` no new smoke failures.

### Phase 6 — Close-out and cooperation (1 subtask)
6.1 Write `docs/reports/WAVE_LOOP_504_CLOSEOUT.md`.
6.2 Write `docs/reports/FPGA_LOOP_COOPERATION_W505_*.md` with three W505 variants.
6.3 Update `docs/NOW.md` and `.trinity/current-issue.md` for W505.
6.4 Commit, update session log / commit count, create `wave-loop-505`.

---

## 3. Risk assessment

| Risk | Mitigation |
|------|-----------|
| Changing loop fuel breaks existing `native_decide` witnesses | Keep `defaultFuel` large (1000); verify W503 for-loop witness still passes. |
| `all_equiv` proof explodes on nested induction | Keep the loop case small and rely on `Valuation.equiv` lemmas; prove a dedicated loop-invariant lemma. |
| `Stmt.isSequential` complicates classifier alignment | The Rust classifier already uses `Stmt.isLowerableFuel`; the new Lean predicate is proof-only. |
| Existing W501/W502 callers break | Keep combinational variants of `module_value_equiv_statement` and add sequential variants; provide implication theorem. |

---

## 4. Acceptance criteria

- `lake build Trinity.IcarusLowerable.Soundness` green, zero `sorry`.
- At least one bounded-loop witness proved with the generic sequential theorem.
- `./scripts/tri verify --lean-lowerable` zero disagreements.
- `./scripts/tri test` no new smoke failures.
- Close-out report and three W505 cooperation variants written.

---

*φ² + φ⁻² = 3 | TRINITY*
