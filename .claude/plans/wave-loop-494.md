# Wave Loop 494 Plan — Semantic equivalence for the Icarus-lowerable scalar subset

**Issue:** #1464 (to create)  
**Branch:** `wave-loop-494`  
**Variant:** A (default) — machine-checked semantic equivalence  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Context and motivation

Wave Loops 491–493 built and hardened the Icarus-lowerability predicate and the
shallow Verilog emitter model in Lean 4. The current contract is syntactic:

```lean
Module.isLowerable env m → ¬ (emitModule env m).hasPlaceholder
```

W493 cleaned up the backend so the predicate and the smoke gate agree (zero
disagreements). The next logical step is a value-preservation theorem: the
modeled Verilog computes the same values as the source t27 program for a
carved-out scalar subset.

This wave deliberately stays within the **scalar numeric/struct subset** that
already lowers to packed bit-vectors. It does not tackle strings, enums, f32, or
dynamic methods.

---

## 2. Goals

1. Define a denotational (or small-step) semantics for the simplified t27 AST
   over concrete bit-vectors for the lowerable subset.
2. Define a matching semantics for the shallow Verilog AST (`VExpr`, `VStmt`).
3. Prove a representative equivalence theorem for at least one W493 positive
   witness (e.g., `w493_nested_struct_field_from_identifier_lowerable`).
4. If time allows, generalize to a parameterized theorem over all modules in
   `Completeness.lean` using `native_decide`.
5. Keep the full repository gate green and produce W494 close-out report +
   W495 cooperation variants.

---

## 3. Decomposed tasks

| # | Task | Owner | Files | Acceptance |
|---|------|-------|-------|------------|
| 1 | Research equivalence-proof precedents | Queen | `.claude/plans/wave-loop-494.md`, update `docs/reports/T27_VS_FORMAL_HDL_*.md` | Precedents and strategy documented. |
| 2 | Define t27 scalar semantics | Creator (C) | `proofs/lean4/Trinity/IcarusLowerable/Semantics.lean` | `Expr` and `Stmt` have computable `eval` functions over bit-vectors. |
| 3 | Define Verilog scalar semantics | Creator (C) | `proofs/lean4/Trinity/IcarusLowerable/Semantics.lean` | `VExpr` and `VStmt` have matching `eval` functions. |
| 4 | Prove representative equivalence | Creator (C) | `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean` or new file | At least one concrete witness module satisfies value preservation. |
| 5 | Generalize with native_decide (stretch) | Creator (C) | `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean` | Add per-module equivalence theorems if feasible. |
| 6 | Verify Lean build | Verifier (V) | `proofs/lean4/` | `lake build Trinity.IcarusLowerable.*` green. |
| 7 | Run conformance and reseal if needed | Verifier (V) | `.trinity/seals/` | `./scripts/tri test --fast --icarus-lowerable` green; reseal only if compiler changes. |
| 8 | Reports and memory | Queen | `docs/reports/WAVE_LOOP_494_CLOSEOUT.md`, `docs/reports/FPGA_LOOP_COOPERATION_W495_*.md`, `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, memory | All documents updated; branch pushed; `wave-loop-495` created. |

---

## 4. Detailed implementation notes

### 4.1 t27 scalar semantics

- Values are `BitVec n` for numeric/bool types. Structs are concatenations of
  leaf-field bit-vectors. Arrays are concatenations of element bit-vectors.
- `Expr.eval` handles literals, identifiers (looked up in an environment),
  binary/unary operators, `fieldAccess` (slice by struct offset), `index`
  (slice by element width), `call` (function call), and `structLit`/`arrayLit`
  (concatenation).
- `Stmt.eval` handles assignment, var/const declaration, if-then-else, for-loop
  (unrolled over a finite range), return, and bare call.

### 4.2 Verilog scalar semantics

- `VExpr.eval` mirrors the t27 semantics: literals, identifiers, binop/unop,
  slice, index, call, concat.
- `VStmt.eval` handles assignment, always-comb, initial, task call, localparam.

### 4.3 Equivalence theorem shape

For a concrete witness module `m` under environment `env`:

```lean
theorem equiv (env : Env) (m : Module) (input : Env) :
  Module.isLowerable env m →
  evalModule t27Sem env m input = evalModule verilogSem env (emitModule env m) input
```

Use `native_decide` on concrete modules. For a generic theorem, keep the proof
obligation decidable and rely on computational equality of finite bit-vectors.

### 4.4 Risks

- Packed-vector slicing for structs must exactly match the Rust emitter's
  field order. Use `widthOfType` and `structFields` from the existing emitter.
- Function-call semantics in Verilog (function returns an expression value)
  differs from t27 `return` statements. Model functions as pure value-returning
  expressions and inline them, matching the emitter's current lowering.
- If the proof does not close within the wave, document the partial result and
  the remaining obligations as a W495 variant.

---

## 5. Definition of done

- [x] `Semantics.lean` defines computable `eval` for t27 (`evalExpr/evalFunction/evalTest`) and Verilog scalar ASTs (`evalVExpr/evalVStmt/evalVModule`).
- [x] Representative equivalence theorem `scalar_struct_value_equiv` proved by
      `native_decide`: the scalar-struct-literal witness computes the same packed
      value in t27 and in the emitted shallow Verilog.
- [x] `lake build Trinity.IcarusLowerable.*` is green.
- [x] `./scripts/tri test --fast --icarus-lowerable` is green (zero disagreements).
- [x] `cargo test -p t27c --bin t27c` is green (1525/0/2).
- [x] No compiler change, no reseal needed.
- [ ] W494 close-out report and W495 cooperation variants are written.
- [ ] `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, and
      persistent memory are updated.
- [ ] `wave-loop-494` is pushed and `wave-loop-495` is created.

---

*φ² + φ⁻² = 3 | TRINITY*
