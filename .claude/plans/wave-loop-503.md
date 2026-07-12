# Wave Loop 503 — Decomposed Plan

**Issue:** #1472  
**Branch:** `wave-loop-503`  
**Variant:** A — extend Icarus equivalence proof to sequential constructs  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Literature / weak-point review

### Weak points closed by W501/W502
- Hard-coded `main` entry point → generalized to any emitted function name.
- Empty argument list → generalized to arbitrary `args : List Value`.

### Remaining weak points
- **Stmt coverage gap:** the operational semantics only models assignment,
  return, and sequential composition. `ifThenElse` and `forLoop` are omitted,
  so the theorem does not apply to realistic t27 specs.
- **Emitter trust gap:** the Verilog emitter handles `if` / `for` in an ad-hoc
  way; without a semantic model there is no formal contract.
- **Predicate over-approximation:** `isCombinationalFuel` may reject
  lowerable sequential statements because it has no rules for them.

### Scientific / engineering references
- CompCert Clight operational semantics (Leroy et al.) — `if` and `for` as
  standard control-flow constructs in a big-step semantics.
- CompCert `RTLtyping` / `RTLgen` — bounded loops as repeated execution.
- Csmith / YARPGen — adversarial generators stress conditional and loop
  constructs; we reuse the idea as hand-written witnesses.
- Icarus Verilog LRM — `if` and `for` are supported inside `always` / `initial`
  and inside generate contexts; our combinational subset maps them to
  `always_comb` / `initial` blocks.

---

## 2. Task decomposition

### Phase 1 — Semantic model (2 subtasks)
1.1 Add `Stmt.ifThenElse` and `Stmt.forLoop` to `SemanticsTotal.lean` on the
    t27 side.
1.2 Add matching evaluation rules on the shallow-Verilog side.

### Phase 2 — Shallow syntax (1 subtask)
2.1 Add `if` / `for` constructors to `Verilog.lean` and their pretty-printing.

### Phase 3 — Emitter update (1 subtask)
3.1 Update `Emitter.lean` so `emitStmt` produces the new constructors when
    the predicate allows them.

### Phase 4 — Lowerability predicate (1 subtask)
4.1 Extend `Predicate.lean` so `ifThenElse` / `forLoop` are lowerable when
    their components are lowerable.

### Phase 5 — Forward-simulation proof (2 subtasks)
5.1 Add `if` case to `all_equiv` / `module_value_equiv_proved`.
5.2 Add `for` induction case to the same proof.

### Phase 6 — Witness specs (1 subtask)
6.1 Write `w503_if_return.t27` and `w503_for_accumulator.t27`.
6.2 Reseal and add `native_decide` value-equivalence theorems.

### Phase 7 — Verification (1 subtask)
7.1 `lake build Trinity.IcarusLowerable.Soundness`.
7.2 `./scripts/tri verify --lean-lowerable`.
7.3 `./scripts/tri test`.

---

## 3. Risk assessment

| Risk | Mitigation |
|------|-----------|
| `for` induction needs fuel accounting | Use the same fuel-based totalization pattern as existing combinational rules; prove a fuel-monotonicity lemma. |
| Icarus `for` inside `always_comb` may not be supported | Restrict `for` to `initial` / testbench contexts or emit generate-style loops where possible. |
| Emitter changes break existing smoke | Add the new rules behind the predicate and run full `./scripts/tri test` before sealing. |
| Lean proof explodes on `if` / `for` cases | Keep witnesses tiny and use `native_decide` for concrete evaluations; keep generic theorem structural. |

---

## 4. Acceptance criteria

- `lake build Trinity.IcarusLowerable.Soundness` green, zero `sorry` in
  IcarusLowerable modules.
- Two new scratch witnesses (`if` and `for`) pass t27 evaluation, Icarus smoke,
  and the Lean value-equivalence theorem.
- `./scripts/tri verify --lean-lowerable` reports zero disagreements.
- `./scripts/tri test` reports no new smoke failures.
- Close-out report and three W504 cooperation variants written.

---

*φ² + φ⁻² = 3 | TRINITY*
