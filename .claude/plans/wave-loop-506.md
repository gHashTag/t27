# Wave Loop 506 — Decomposed Plan

**Issue:** #1475 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-506`  
**Variant:** B — model `switch` expressions for scalar dispatch  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Extend the Icarus-lowerable operational semantics, shallow Verilog model,
emitter, predicate, and generic equivalence theorem to cover t27 `switch`
expressions. Target at least one integer/bool scratch witness that passes both
the classifier and Icarus smoke and has a value-preservation theorem.

**Scope note:** true enum/trit dispatch is reduced to the scalar-dispatch case in
this wave. The Rust Verilog backend currently emits enum variants as bare
identifiers (e.g. `uninitialized`) while declaring `localparam EnumName_uninitialized`,
so an enum-dispatch witness would fail Icarus elaboration. W506 therefore
hard-ens the model with **integer (and bool) switches** first; enum variant
encoding becomes a residual boundary for W507.

---

## Scientific / engineering anchors

- **CompCert Clight** — structured `switch` with mandatory `default` last,
  `select_switch` case selection, and fuel-bounded reference interpreter
  (`exec_stmt(W, n, ...)`). ([Blazy & Leroy, 2009](https://xavierleroy.org/publi/Clight.pdf))
- **Icarus / SystemVerilog case semantics** — procedural `case` as
  select-one-of-many with `default`; `unique case` asserts exactly one match.
  ([Cummings, "full_case parallel_case"](https://csg.csail.mit.edu/6.375/6_375_2007_www/papers/cummings-case-snug99.pdf))
- **Csmith / YARPGen** — adversarial compiler fuzzing with switch-like dispatch
  and static UB avoidance. ([Yang et al., PLDI 2011](https://doi.org/10.1145/1993316.1993532))
- **Kami / Bluespec / Kôika** — rule-based hardware semantics with `IfElse`
  dispatch and trace refinement; switch expressions map naturally to nested
  conditional expressions. ([Kami ICFP 2017](https://adam.chlipala.net/papers/KamiICFP17/KamiICFP17.pdf), [Kôika PLDI 2020](https://people.csail.mit.edu/bthom/pldi20.pdf))

---

## Weak-point analysis

1. **Branch topology blocker.** `wave-loop-506` was created from `master`, which
   does not contain the W491–W505 Icarus-lowerability stack. The target files
   (`Ast.lean`, `Verilog.lean`, `SemanticsTotal.lean`, `Emitter.lean`,
   `Predicate.lean`, `Equivalence.lean`) do not exist on this branch yet.
2. **Enum variant encoding bug.** The Rust backend declares
   `localparam EnumName_variant = value` but references enum values as bare
   `variant` in generated switch expressions, so enum dispatch is not
   elaboration-clean in Icarus.
3. **No expression-level conditional in the shallow Verilog model.** The Rust
   backend lowers switch expressions to nested ternary operators; the model must
   gain a corresponding `VExpr.ternary` constructor.
4. **Cross-cut surface.** Adding one expression constructor touches Ast,
   Verilog, SemanticsTotal, Emitter, Predicate, Equivalence, AstInduction,
   Lemmas, and Soundness.
5. **`Expr.typeOfFuel` does not cover switch or enum values.** Type inference for
   the new construct must return a deterministic type.

---

## Phases

### 1. OBSERVE — confirm boundaries
- Read `.trinity/current-issue.md` and `docs/reports/FPGA_LOOP_COOPERATION_W506_2026-07-07.md`.
- Replay W505: `Lemmas.lean` / `Soundness.lean` witness shape and
  `module_value_equiv_proved_sequential` usage.
- Verify the branch gap: `git merge-base wave-loop-506 wave-loop-505`.

### 2. TOPOLOGY FIX — inherit W505 work
- Rebase `wave-loop-506` onto `wave-loop-505` so the Icarus-lowerability file
  tree is present.
- Re-run `./scripts/tri test` and `./scripts/tri verify --lean-lowerable` to
  re-establish the 711/711 green baseline.

### 3. SPEC — design the switch model
- Add `Expr.switch (discriminant : Expr) (cases : List (Expr × Expr)) (default : Expr)`
  to the simplified t27 AST.
- Add `VExpr.ternary (cond : VExpr) (then_ else_ : VExpr)` to the shallow
  Verilog AST and wire it into `hasPlaceholder`.
- Define semantics: evaluate `discriminant`; run the first case whose tag equals
  the discriminant; otherwise run `default`. No fall-through.
- Design witnesses:
  - `w506_switch_int.t27` — `switch (x) { 0 => 10, 1 => 20, 2 => 30, else => 99 }`.
  - `w506_switch_bool.t27` — `switch (flag) { true => 1, false => 0, else => 0 }`.
  - Optional `w506_switch_trit.t27` — `switch (t) { -1 => 1, 0 => 2, 1 => 3, else => 0 }`
    to exercise trit-like dispatch without needing enum encoding.

### 4. TDD — write the spec before the model
- Add scratch spec(s) with `test` blocks covering all arms and the default.
- Run `./scripts/tri test` to capture the current baseline (likely the Rust
  backend already emits a nested ternary and smoke passes, while the Lean model
  would reject the construct if it were represented).

### 5. CODE/IMPL — extend every layer
- `proofs/lean4/Trinity/IcarusLowerable/Ast.lean`
  - Add `Expr.switch` constructor.
- `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`
  - Add `VExpr.ternary` and update `hasPlaceholder`.
- `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`
  - Add `evalExprTotal` case for `Expr.switch`.
  - Add `evalVExprTotal` case for `VExpr.ternary`.
- `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`
  - Emit `Expr.switch` as nested `VExpr.ternary` chains.
- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
  - Lowerability: discriminant and every arm lowerable.
  - Combinationality: discriminant and every arm combinational.
  - Type inference: type of switch = type of `default` (arms must agree).
  - Call context: collect calls from discriminant and all arms.
- `proofs/lean4/Trinity/IcarusLowerable/AstInduction.lean`
  - Update structural induction helpers if they enumerate `Expr` constructors.
- `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`
  - Add simplification lemmas: `emitExpr_default_switch`,
    `evalExprTotal_succ_switch`, `evalVExprTotal_succ_ternary`.
  - Add the `switch` case to the `all_equiv` expression induction.

### 6. GEN — inspect emitted Verilog
- Run `./scripts/tri gen` for the new witness(es).
- Verify the emitted Verilog contains only scalar ternary / comparison patterns,
  no `UNSUPPORTED_ICARUS` or `// TODO` placeholders.

### 7. SEAL — save deterministic hashes
- Run `t27c seal <spec> --save` for each new scratch spec.

### 8. VERIFY — prove and run gates
- Add W506 witness environments/modules in `Lemmas.lean`.
- Add lowerability / sequentiality / value-preservation theorems in
  `Soundness.lean`, applying `module_value_equiv_proved_sequential` for at
  least one witness.
- `lake build Trinity.IcarusLowerable.Soundness`.
- `./scripts/tri verify --lean-lowerable` — must report zero disagreements.
- `./scripts/tri test` — must keep 711/711 non-smoke, 191/191 yosys/Icarus,
  zero baseline failures.
- `cargo test -p t27c --bin t27c` — must stay 1525 / 0 / 2.

### 9. LAND — commit and hand off
- Commit to `wave-loop-506` with `Closes #1475`.
- Update `.trinity/current-issue.md`, `docs/NOW.md`,
  `.trinity/current_task/.commit_count`, and `session_log.jsonl`.
- Create `wave-loop-507` branch.

### 10. LEARN — capture experience
- Save new patterns (cross-AST expression constructor, nested ternary emission,
  branch topology fix, enum encoding residual) to `.trinity/experience.md` and
  persistent memory.

---

## Acceptance criteria

- `wave-loop-506` contains the W505 Icarus-lowerability stack.
- At least two new scratch switch witnesses pass both the classifier and
  Icarus smoke.
- `lake build Trinity.IcarusLowerable.Soundness` is green with zero `sorry`.
- `./scripts/tri verify --lean-lowerable` reports zero disagreements.
- `./scripts/tri test` keeps zero baseline failures.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundaries for W507

- True enum dispatch: needs `Env` variant-value table and a Rust backend fix to
  reference `EnumName_variant` in `ExprEnumValue` / `ExprSwitch`.
- `while` loops remain outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).

---

*φ² + φ⁻² = 3 | TRINITY*
