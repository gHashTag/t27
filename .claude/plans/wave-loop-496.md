# Wave Loop 496 — Decomposed Plan

**Issue:** #1466  
**Branch:** `wave-loop-496`  
**Variant:** A — prove the generic structural equivalence theorem  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Weak points identified

1. `module_value_equiv_statement` is a `sorry`. It promises equivalence for any
   `Module.isLowerable` module but is not proved.
2. `Module.isLowerable` currently accepts `ifThenElse` and `forLoop`, which the
   Verilog evaluator does not model operationally. The generic theorem must either
   exclude them or add a semantics for them.
3. `Expr.typeOf` is a heuristic partial function. It does not track local
   variable types inside function bodies, so a generic expression-equivalence
   lemma must rely on the valuation invariant carrying enough type information
   or on the fact that emitted identifiers are sliced/used in context that
   already knows their width.
4. The valuation invariant between t27 and Verilog is not defined. We need a
   precise relation that is preserved by statement evaluation and by parameter
   binding.
5. Mutual recursion in `evalExpr` / `evalVExpr` / `evalStmts` / `evalVStmts` makes
   structural induction awkward; the proof must be organized around the AST
   constructors rather than around the evaluator mutual block.
6. The emitter does not guarantee that every identifier in the emitted Verilog
   is initialized before use. The model uses an initial valuation of `none`, so
   the invariant must require that every used identifier is bound before the
   point of use.

---

## Scientific background

- **CompCert** (Xavier Leroy et al., 2006–present). A verified C compiler in Coq.
  Its core proof strategy is forward simulation per pass, composed transitively,
  then converted to backward simulation when the target is deterministic.
  Relevant because our single-pass source-to-shallow-Verilog translation is a
  tiny instance of the same simulation idea: source execution implies target
  execution.
  - Source: [CompCert `driver/Compiler.v`](https://github.com/AbsInt/CompCert/blob/master/driver/Compiler.v)
  - Paper: [A formally verified compiler back-end (JAR 2009)](https://www.cs.cmu.edu/~15811/papers/compcert-journal.pdf)
- **Coq'Art** (Yves Bertot & Pierre Castéran, 2004). The standard reference for
  proving properties of inductive types and recursive functions in Coq. Chapter 14
  covers induction principles and recursors; the methodology applies directly to
  Lean 4 structural induction.
  - Source: [Coq'Art home page](https://www.labri.fr/perso/casteran/CoqArt/index.html)
  - Chapter 14: [Foundations of Inductive Types](https://www.labri.fr/perso/casteran/CoqArt/chapter14.pdf)
- **Lean 4 functional induction** (`Lean.Meta.Tactic.FunInd`). Lean can derive
  induction principles from structurally recursive functions. For our purposes,
  manual structural induction over the AST constructors is clearer because the
  evaluators are `partial` and operate inside `Option`.
  - Source: [Recursive Definitions reference](https://lean-lang.org/doc/reference/latest/Definitions/Recursive-Definitions/)
  - Source: [FunInd API](https://lean-lang.org/doc/api/Lean/Meta/Tactic/FunInd.html)
- **Translation validation** (George Necula, PLDI 2000). Validates compiler
  transformations without verifying the compiler itself. Our `native_decide`
  witness proofs are translation-validation checks; the generic theorem is the
  unbounded complement.
  - Source: [Translation Validation for an Optimizing Compiler](http://people.eecs.berkeley.edu/~necula/Papers/tv_pldi00.pdf)
- **Alive2** (Lopes et al., PLDI 2021). Bounded translation validation for LLVM
  IR using SMT. Shows that automatic equivalence checking can scale to real
  compilers and find bugs.
  - Source: [Alive2 paper](https://dl.acm.org/doi/10.1145/3453483.3454030)

---

## Decomposition

### Phase 1 — Restrict the subset (1 h)

- Add `Function.isCombinational` and `Module.isCombinational` predicates that
  require all reachable function/test/bench bodies to contain only
  `assign`, `varDecl`, `constDecl`, `return_`, and `bareCall`.
- Add `Expr.isCombinational` as a sanity check (no nested unsupported nodes).
- Keep `Module.isLowerable` unchanged so the existing gate and witness theorems
  still compile; the generic theorem will use `isLowerable ∧ isCombinational`.

### Phase 2 — Valuation invariant (1 h)

- Define `Valuation.equiv (env : Env) (t27val verilogval : Valuation) : Prop`
  as:
  - For every identifier `x` whose type in `env.vars` or function parameters is
    lowerable, `t27val x = verilogval x` (same `Value`).
  - `__return` is optional and may be absent on both sides.
- Prove trivial lemmas: `equiv` is preserved by updating a single binding with
  equal values; initial all-`none` valuations are equivalent.
- Prove parameter-binding equivalence: zipping equal arg lists produces equivalent
  parameter valuations on top of a base valuation.

### Phase 3 — Expression equivalence (2–3 h)

For each lowerable expression constructor `e`, prove
```lean
Valuation.equiv env v27 vv →
  evalExpr env m v27 e = evalVExpr env (emitModule env m) vv (emitExpr env m e)
```
by structural induction on `e`.

Cases:
- `boolLit`, `intLit`: both evaluators return the same literal value.
- `identifier`: follows directly from the valuation invariant.
- `binop`, `unop`: induction hypothesis + the fact that `evalBinop`/`evalUnop`
  are shared between both evaluators.
- `fieldAccess`: induction hypothesis + the fact that `Expr.typeOf` computes the
  same struct name on both sides (the emitter uses `Expr.typeOf env m`, the
  evaluator uses it too) and slicing uses the same offset/width.
- `index`: induction hypothesis + element width derived from `Expr.typeOf`.
- `call`: induction hypothesis for args + equivalence of function inlining
  (covered in Phase 4).
- `structLit`, `arrayLit`: induction hypothesis + `Value.concatList`.

Because the evaluators are `partial` and in `Option`, the lemma should be stated
with `Option.map` or by case analysis on `evalExpr`. The cleanest formulation is
a relation `equivExprResult` that relates two `Option Value`s.

### Phase 4 — Statement-list equivalence (1–2 h)

Prove that for a list of combinational statements,
```lean
Valuation.equiv env v27 vv →
  Option.map (evalStmts env m v27 stmts) (fun v27' => ... ) =
  Option.map (evalVStmts env (emitModule env m) vv (emitStmts env m stmts)) (fun vv' => ... )
```
or, equivalently, that the resulting valuations are equivalent if both evaluate
successfully.

Handle each statement form:
- `assign` / `varDecl` / `constDecl`: use the expression-equivalence lemma and
  the update-preserves-equiv lemma.
- `return_`: use expression equivalence and the fact that both sides write
  `__return`.
- `bareCall`: combinational task call in test blocks; treat as no-op in the
  model (consistent with current `evalVStmt`).
- Other forms excluded by `isCombinational`.

### Phase 5 — Function inlining equivalence (1 h)

Combine Phase 4 with parameter binding:
```lean
evalFunction env m fn args base = evalVFunction env (emitModule env m) (emitVFunction env m fn) args base'
```
when `base` and `base'` are equivalent and the arg lists are pairwise equal.

This covers `evalCall` and `evalVExpr .call`.

### Phase 6 — Module-level theorem (1 h)

- Prove that evaluating module globals preserves valuation equivalence.
  Globals are `constDecl` / `varDecl` / `assign`; covered by Phase 4.
- After globals, run `main` using Phase 5.
- Conclude `evalModuleFunction env m "main" [] = evalVModule env (emitModule env m) "main"`.

### Phase 7 — Validation (1 h)

- `lake build` of the IcarusLowerable modules.
- `cargo test -p t27c --bin t27c`.
- `./scripts/tri test --fast`.

### Phase 8 — Close-out and W497 variants (1 h)

- `docs/reports/WAVE_LOOP_496_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W497_2026-07-13.md`
- Update `docs/NOW.md`, `.trinity/experience.md`, persistent memory.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| `partial` evaluators block structural induction | State lemmas over AST constructors, use `native_decide` for concrete base cases, and accept that the proof is manual rather than deriving `funind` from partial defs |
| `Expr.typeOf` does not track local vars | Include local variable types in the valuation invariant, or restrict the generic theorem to modules where all used identifiers have `env.vars` entries |
| `ifThenElse` / `forLoop` are lowerable but not modeled | Exclude them via `isCombinational`; document that the generic theorem covers the combinational subset only |
| `bareCall` in test blocks | Model it as no-op in both evaluators; the existing witness theorems still pass |
| Proof gets too long for one wave | Prove the expression-equivalence lemma first and leave statement-level module-level lemmas as follow-up; do not let perfect block good |

---

*φ² + φ⁻² = 3 | TRINITY*
