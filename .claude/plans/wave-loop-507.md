# Wave Loop 507 — Decomposed Plan

**Issue:** #1476 (placeholder — GitHub token still failing)  
**Branch:** `wave-loop-507`  
**Variant:** A — bounded `while` loops in the Icarus-lowerable subset  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Model bounded `while` loops in the Icarus-lowerable operational semantics, the shallow Verilog model, the emitter, the lowerability/sequential predicate, and the generic equivalence theorem. The loop is **fuel-bounded**: each iteration consumes one fuel unit, the combinational condition is re-evaluated at every step, and execution stops when the condition is false or fuel runs out.

Target at least one scratch witness that passes both the Rust Icarus-lowerability classifier and Icarus smoke simulation, has a deterministic seal, and has a value-preservation theorem proved via `module_value_equiv_proved_sequential`.

---

## Scientific / engineering anchors

- **CompCert / Clight — fuel-based big-step semantics for loops.**  
  Leroy & Blazy use a step-indexed fuel `n` to give a total big-step semantics to C-like loops; the t27 `forLoop` already follows the same discipline (one fuel unit per iteration, body at smaller fuel). This is the direct template for `while`.  
  ([Blazy & Leroy, 2009](https://xavierleroy.org/publi/Clight.pdf))

- **CakeML — functional big-step semantics with an explicit clock.**  
  Kumar, Myreen, Owens et al. define a clocked evaluator that returns `Timeout` when the clock expires, analogous to the `Option.none` fuel-exhausted behaviour in the t27 model. Their proof style is the reference for showing that a source loop and its emitted target loop agree under the same clock.  
  ([Kumar et al., ICFP 2013](https://doi.org/10.1145/2500365.2500601))

- **Kami — rule-based hardware semantics with bounded atomic actions.**  
  Choi et al. model hardware as guarded atomic actions in Coq; each rule execution is a bounded state update. The `while` design treats every loop iteration as an atomic sequential state transformer, fitting the Kami discipline.  
  ([Choi et al., PLDI 2017](https://adam.chlipala.net/papers/KamiPLDI17/KamiPLDI17.pdf))

- **IEEE 1800 SystemVerilog procedural loop semantics.**  
  `while (cond) begin … end` inside a function/task or `initial` block re-evaluates `cond` after the body, exactly as the shallow `VStmt.whileLoop` will be defined. The Icarus smoke gate validates that the emitted Verilog behaves this way.

---

## Weak-point analysis

1. **`while` is not in the simplified Icarus AST.**  
   `proofs/lean4/Trinity/IcarusLowerable/Ast.lean` has `Stmt.forLoop` and `Stmt.switch`, but no `Stmt.whileLoop`. The Verilog AST (`Verilog.lean`) and the total semantics (`SemanticsTotal.lean`) also lack it.

2. **Termination is dynamic, not structural.**  
   `forLoop` recurses on a concrete `Nat` bound in addition to fuel, so the equivalence proof has two decreasing measures. `while` recurses only on fuel; the proof must be careful that the condition evaluates identically on both sides at every iteration.

3. **Condition re-evaluation after a mutating body.**  
   In the source semantics the condition is evaluated against the valuation produced by the body. The emitted Verilog `while (cond) begin body end` must do the same. Any mismatch in assignment sequencing or local-declaration semantics breaks equivalence.

4. **The predicate and classifier may disagree on `while`.**  
   The Rust Verilog backend already emits `while`, but the Icarus-lowerability classifier may not inspect `StmtWhile` deeply enough. The formal predicate, the Rust classifier, and the emitted Verilog must accept/reject the same specs.

5. **`Stmt.isCombinational_implies_isSequential` enumerates every statement constructor.**  
   Adding `whileLoop` requires a new contradiction branch because `whileLoop` is never combinational.

6. **`module_value_equiv_proved_sequential` is the witness entry point.**  
   The generic `all_equiv` theorem must be extended with a new `P_whileLoop` predicate and a matching proof case; otherwise `Soundness.lean` cannot apply the theorem to while-bearing witnesses.

---

## Design decisions

- **Fuel-bounded, no surface-syntax change.**  The t27 parser already supports `while (cond) { body }`; the Icarus model treats fuel as the iteration cap. This is the smallest change that closes the gap and matches the existing `forLoop` fuel discipline.
- **Condition is combinational.**  A `whileLoop` is sequential only when `cond.isCombinational'` holds and the body is sequential. The condition is evaluated at the same fuel as the loop step (one unit below the outer `fuel + 1`).
- **No separate static bound expression.**  A future wave can add a source-level bound annotation if synthesizability requires a guaranteed compile-time cap.

---

## Phases

### 1. OBSERVE — confirm boundaries and patterns
- Re-read `.trinity/current-issue.md` and `docs/reports/FPGA_LOOP_COOPERATION_W507_2026-07-07.md`.
- Read `proofs/lean4/Trinity/IcarusLowerable/Ast.lean`, `Verilog.lean`, `SemanticsTotal.lean`, `Predicate.lean`, `Emitter.lean`, `Equivalence.lean`, `Lemmas.lean`, `Soundness.lean` around the `forLoop` case.
- Read `bootstrap/src/compiler.rs` around `StmtWhile`, `gen_verilog_while_stmt`, and `IcarusAnalyzer` lowerability checks.
- Verify topology: `wave-loop-507` includes `wave-loop-506` (`git merge-base wave-loop-507 wave-loop-506`).
- Run `./scripts/tri test` and `lake build Trinity.IcarusLowerable.Soundness` to capture the W506 green baseline.

### 2. SPEC — design the bounded-while model
- Add `Stmt.whileLoop (cond : Expr) (body : List Stmt)` to the simplified AST.
- Add `VStmt.whileLoop (cond : VExpr) (body : List VStmt)` to the shallow Verilog AST.
- Define total semantics:
  - `evalWhileLoopTotal fuel env m val cond body`
  - `evalVWhileLoopTotal fuel env vm val cond body`
  - Each iteration consumes one fuel unit; condition evaluated at the smaller fuel, body evaluated at the smaller fuel, then recurse.
- Define predicate rules:
  - Not combinational.
  - Lowerable when `cond` is lowerable and `body` is lowerable.
  - Sequential when `cond` is combinational and `body` is sequential.
- Design scratch witnesses:
  - `w507_while_counter.t27` — count up while `i < n`, return final count.
  - `w507_while_search.t27` — linear search over a fixed array until a match.
  - `w507_while_nested.t27` — `while` inside a bounded `for`.

### 3. TDD — write the specs before the model
- Add the three scratch specs with `test`/`invariant` blocks.
- Run `./scripts/tri test` to capture the current baseline (the specs should parse and the Rust backend should emit `while`, but they will not be marked Icarus-lowerable yet).
- Record expected failures/baselines.

### 4. CODE/IMPL — extend every layer

**Lean 4 model**
- `proofs/lean4/Trinity/IcarusLowerable/Ast.lean`: add `Stmt.whileLoop` constructor.
- `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`: add `VStmt.whileLoop`; update `VStmt.hasPlaceholder`.
- `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`: add `evalWhileLoopTotal` / `evalVWhileLoopTotal`; add cases to `evalStmtTotal` / `evalVStmtTotal`.
- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`:
  - `Stmt.isLowerableFuel` / `Stmt.isCombinationalFuel` / `Stmt.isSequential'` / `Stmt.isCombinational'` cases.
  - `Stmt.functionNamesFuel` / `Stmt.functionNames'` cases.
  - Update the `Stmt.isCombinational_implies_isSequential` proof for the new contradiction branch.
- `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`: add `emitStmt` case for `Stmt.whileLoop`.
- `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`:
  - Add helper lemmas: `emitStmt_default_whileLoop`, `evalStmtTotal_succ_whileLoop`, `evalVStmtTotal_succ_whileLoop`.
  - Add `Stmt.callContext_whileLoop` / `Stmt.isSequential_whileLoop` decomposition lemmas in the helper section.
  - Define `P_whileLoop` next to `P_forLoop`.
  - Extend the `all_equiv` tuple to include `P_whileLoop` and add its zero / succ proofs.
  - Add the `Stmt.whileLoop` branch in the `P_stmt` proof.
- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`: add `w507WhileCounterEnv` / `Module` / `Function` definitions.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`: prove `w507_while_counter_lowerable`, `w507_while_counter_sequential`, `w507_while_counter_value_equiv` via `module_value_equiv_proved_sequential`.

**Rust classifier**
- Inspect `bootstrap/src/compiler.rs` (`IcarusAnalyzer`, `fn_body_has_unlowerable_construct`, `icarus_lowerable_violations`).
- Ensure `StmtWhile` is checked for lowerability exactly like `StmtFor`: condition and body must be lowerable, and the loop context is scanned.
- If necessary, add a specific `StmtWhile` branch to lowerability predicates.

### 5. GEN — inspect emitted Verilog
- Run `./scripts/tri gen` for each new scratch spec.
- Verify generated Verilog contains procedural `while (cond) begin … end` and no `UNSUPPORTED_ICARUS` / `// TODO` placeholders inside the loop.

### 6. SEAL — save deterministic hashes
- Run `t27c seal <spec> --save` for each new scratch witness.

### 7. VERIFY — prove and run gates
- `lake build Trinity.IcarusLowerable.Soundness` — must be green with zero `sorry`.
- `./scripts/tri verify --lean-lowerable` — must report zero lowerability disagreements.
- `./scripts/tri test` — must keep zero new baseline failures; all non-smoke, yosys smoke, and Icarus smoke green.
- `cargo test -p t27c --bin t27c` — must stay at the current baseline (1525 / 0 / 2 at W506).
- If any gate regresses, fix the offending layer and re-verify.

### 8. LAND — commit and hand off
- Commit all changes to `wave-loop-507` with `Closes #1476`.
- Update `.trinity/current-issue.md`, `.trinity/current_task/.commit_count`, and `session_log.jsonl` for Wave Loop 507 close-out and Wave Loop 508 setup.
- Create branch `wave-loop-508`.

### 9. LEARN — capture experience
- Write `docs/reports/WAVE_LOOP_507_CLOSEOUT.md` summarising weak points, scientific anchors, what changed, verification, and residual boundaries.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W508_2026-07-07.md` with three candidate directions for Wave Loop 508.
- Save the learnings to persistent memory (`memory/wave-loop-507.md`) and update `MEMORY.md`.
- (Skills are saved at the end of the wave per the standing charter.)

---

## Acceptance criteria

- `Stmt.whileLoop` and `VStmt.whileLoop` exist in the simplified ASTs.
- `evalWhileLoopTotal` / `evalVWhileLoopTotal` are total, fuel-bounded, and re-evaluate the condition each iteration.
- The lowerability predicate accepts `while` when its condition and body are lowerable; the sequential predicate requires a combinational condition.
- The emitter maps `Stmt.whileLoop` to `VStmt.whileLoop`.
- `all_equiv` includes `P_whileLoop` and the theorem builds with zero `sorry`.
- At least one new scratch witness:
  - parses and passes `./scripts/tri test`,
  - is sealed,
  - passes the Icarus-lowerability classifier,
  - has a value-preservation theorem in `Soundness.lean`.
- `./scripts/tri test` has zero new baseline failures.
- `cargo test -p t27c --bin t27c` matches the W506 baseline.

---

## Residual boundaries for W508

- True enum dispatch in `switch` remains a Rust/Lean encoding boundary.
- Array-typed direct fields still fall back to memory-mode lowering.
- `while` loops have no source-level static bound annotation; fuel is the only iteration cap.
- Non-deterministic / wildcard pattern matching in `switch` is not modeled.
- The equivalence theorem still requires the chosen function to be emitted and non-host-only.

---

## Next-wave cooperation variants (W508)

### Variant A — Source-level static loop bounds (default recommendation)
Add an optional bound annotation to t27 loops, e.g. `while (cond) : (max n) { … }`, and propagate the bound into the Lean model so the equivalence proof can reason about a concrete iteration cap independent of fuel. This closes the synthesizability gap and makes the Icarus subset attractive to hardware designers who need guaranteed termination.

### Variant B — Harden `switch` / enum dispatch
Align the Rust backend's enum `localparam` naming with the Lean `Env.enumValue` model, add support for `switch` default arms and nested `switch` in expression position, and prove the new cases. This is lower risk than static loop bounds and keeps the W506 proof machinery warm.

### Variant C — Array-typed direct fields / memory-mode lowering
Remove the memory-mode fallback for struct fields that are fixed-size scalar arrays. Add a shallow Verilog model for packed-vector field access and prove value preservation for read/write/parameter/return paths. This closes a long-standing lowering boundary but touches the struct/array intersection, which has been a recurring source of packing/name-collision bugs.

---

*φ² + φ⁻² = 3 | TRINITY*
