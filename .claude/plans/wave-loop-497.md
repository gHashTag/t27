# Wave Loop 497 — Decomposed Plan

**Issue:** #1467
**Branch:** `wave-loop-497`
**Variant:** A — totalize the Icarus-lowerable combinational evaluator and prove
            the generic structural equivalence theorem
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Weak points identified

1. **The generic theorem `module_value_equiv_statement` is a `sorry`.**
   Root cause: `evalExpr`, `evalVExpr`, `evalStmts`, and `evalVStmts` in
   `Semantics.lean` are `partial` mutual definitions, which Lean marks as unsafe
   and refuses to unfold in proofs.
2. **Default zero widths in `varDecl` / `constDecl` disagree between t27 and
   Verilog semantics.**
   t27 uses `⟨1, 0#1⟩` for uninitialized declarations, while the emitter produces
   `VExpr.lit width "0"` where `width = widthOfType env ty`. A generic equivalence
   proof must align these defaults.
3. **`return_ none` is semantically misaligned.**
   t27 leaves `__return` unbound, while the emitter assigns `__return = 1'b0`.
   The combinational subset should require an explicit `return_ (some e)`.
4. **No total evaluator exists.**
   All proof-relevant evaluation functions are `partial`. A fuel-based total
   evaluator is needed both for the generic theorem and for bridge lemmas to the
   existing witness set.
5. **The theorem statement does not restrict the subset.**
   `Module.isLowerable` alone accepts `ifThenElse` and `forLoop`, which are not
   modeled operationally. The theorem must also require `Module.isCombinational`.

---

## Scientific background

- **CompCert** (Leroy et al., 2006–present). Verified C compiler in Coq.
  Proves compiler correctness by composing forward simulations and converting
  to backward simulations for deterministic targets. Our single-pass shallow
  t27 → Verilog translation is a tiny instance: a direct forward simulation
  (source value implies target value) is sufficient because the emitted
  Verilog evaluator is deterministic.
  - Source: [CompCert `driver/Compiler.v`](https://github.com/AbsInt/CompCert/blob/master/driver/Compiler.v)
  - Paper: [Formal Verification of a Realistic Compiler — CACM](https://cacm.acm.org/research/formal-verification-of-a-realistic-compiler/)
  - Paper: [An Inductive Proof Method for Simulation-based Compiler Correctness](https://ar5iv.labs.arxiv.org/html/1611.09606)
- **Fuel-based total semantics**. Standard technique for making partial
  functions proof-transparent in dependent type theory: thread a `fuel : Nat`
  parameter that decreases on each recursive call and return an error value
  when exhausted. Used in Lean4Lean, Template-Coq/MetaCoq, and the MiniRadix
  tutorial.
  - Source: [Lean4Lean paper (arXiv 2403.14064)](https://doi.org/10.48550/arxiv.2403.14064)
  - Source: [MiniRadix InterpCorrectness.lean](https://github.com/leodemoura/ETAPSTutorial2026/blob/main/MiniRadix/Proofs/InterpCorrectness.lean)
  - Source: [Fueled Evaluation for Decidable Type Checking](https://hirrolot.github.io/posts/fueled-evaluation.html)
- **Lean 4 `partial` vs fuel / `partial_fixpoint`.**
  `partial` definitions are computable but cannot appear in proof terms.
  `partial_fixpoint` (PR #6355) supports equational reasoning for tail-recursive
  / monadic partial functions via domain theory. For our AST interpreter, the
  fuel pattern is simpler and gives a total function directly.
  - Source: [lean4.dev termination proofs](https://lean4.dev/tactics/automation/termination)
  - Source: [lean4 PR #6355 partial_fixpoint](https://github.com/leanprover/lean4/pull/6355)
  - Source: [lean4 PR #7965 Nat fix-operator](https://github.com/leanprover/lean4/pull/7965)
- **Translation validation** (Necula, PLDI 2000; Alive2, PLDI 2021). Bounded
  equivalence checking for concrete inputs. Our `native_decide` witness proofs
  are a proof-assistant analog; the generic theorem is the unbounded complement.

---

## Decomposition

### Phase 1 — Tighten the combinational subset predicate (1 h)

- In `Predicate.lean`, update `Stmt.isCombinational` so that:
  - `assign lhs rhs` requires both sides combinational.
  - `varDecl` / `constDecl` require an explicit initializer that is combinational.
  - `return_` requires `some e` with `e` combinational.
  - `bareCall e` requires `e` combinational.
  - All other constructs are not combinational.
- This guarantees that a combinational function always returns a value and that
  every declared variable/constant has a well-defined, lowerable initial value.

### Phase 2 — Align default declaration semantics (1 h)

- In `Semantics.lean`, change the `none` branches of `varDecl` and `constDecl`
  to produce a zero value whose width is `widthOfType env ty`, matching the
  emitter's `VExpr.lit width "0"`. This only affects specs with uninitialized
  declarations; all current witnesses have explicit initializers.

### Phase 3 — Implement fuel-based total evaluator (3 h)

- Create `SemanticsTotal.lean`.
- Define mutually recursive total functions parameterized by `fuel : Nat`:
  - `evalExprTotal fuel env m val e`
  - `evalVExprTotal fuel env vm val e`
  - `evalStmtsTotal fuel env m val stmts`
  - `evalVStmtsTotal fuel env vm val stmts`
  - `evalFunctionTotal fuel env m fn argVals base`
  - `evalVFunctionTotal fuel env vm fn argVals base`
- Fuel decreases on every nested call / statement-step. Out of fuel returns
  `none`.
- Define size functions `Expr.size`, `Stmt.size`, `Function.size`, `Module.size`
  and use them to provide default-fuel wrappers:
  - `evalModuleFunctionTotal env m fnName args`
  - `evalVModuleTotal env vm fnName`
- Prove a determinism / fuel-monotonicity lemma: if the evaluator succeeds with
  fuel `n`, it succeeds with the same result for any `m ≥ n`.

### Phase 4 — Prove generic structural equivalence theorem (4 h)

- State the theorem using total evaluators and the combinational subset:
  ```
  theorem module_value_equiv_statement (env : Env) (m : Module)
      (h : Module.isLowerable env m)
      (hcomb : Module.isCombinational env m)
      (mainFn : Function)
      (hm : m.findFunction "main" = some mainFn) :
      evalModuleFunctionTotal env m "main" [] =
      evalVModuleTotal env (emitModule env m) "main" := by ...
  ```
- Prove expression equivalence by structural induction over `Expr` using the
  custom recursor from `AstInduction.lean`. Key cases:
  - Literals and identifiers are identical by construction.
  - `binop` / `unop` follow from the induction hypothesis and `evalBinop` /
    `evalUnop` being shared.
  - `fieldAccess` / `index` follow from type-derived widths and offsets.
  - `call` follows from the function-equivalence induction hypothesis.
  - `structLit` / `arrayLit` follow from list-concatenation preservation.
- Prove statement-list equivalence by list induction; the combinational subset
  guarantees only `assign`, `varDecl`, `constDecl`, and `return_ (some e)` occur.
  Show each construct updates `Valuation.equiv` valuations identically.
- Lift to function inlining and module globals.

### Phase 5 — Bridge total and partial evaluators on witnesses (1 h)

- For each W495 witness (scalar_struct, w493_nested_identifier,
  w493_local_scalar, w493_module_scalar, w493_module_aos), add a `native_decide`
  lemma of the form:
  ```
  evalModuleFunctionTotal env witness "main" [] =
  evalModuleFunction env witness "main" []
  ```
  and similarly for the Verilog side.
- Keep the original W495 witness value-equivalence theorems untouched as
  regression tests.

### Phase 6 — Verification and close-out (2 h)

- `lake build` all IcarusLowerable modules with zero `sorry`.
- `./scripts/tri test --fast`.
- `cargo test -p t27c --bin t27c`.
- Write `docs/reports/WAVE_LOOP_497_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W498_*.md`.
- Update `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`.
- Save persistent memory entry for W497.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| Fuel-based evaluator diverges from partial evaluator on a witness | Add `native_decide` bridge lemmas and fix any discrepancy before closing |
| Structural proof on nested `Expr` is unwieldy | Reuse `AstInduction.lean`; prove expression equivalence as a separate lemma first |
| `Stmt.isCombinational` change breaks existing theorems | Only restrict `return_` and require explicit init; current witnesses satisfy this |
| Verilog `localparam` extraction semantics complicates the invariant | Keep current semantics and require constDecl init to be exactly the emitted width |

---

*φ² + φ⁻² = 3 | TRINITY*
