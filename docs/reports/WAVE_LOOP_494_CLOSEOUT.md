# Wave Loop 494 Close-Out Report

**Issue:** #1464 (closed by this wave)  
**Branch:** `wave-loop-494`  
**Variant selected:** A — machine-checked semantic equivalence for the Icarus-lowerable scalar subset  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. What was attempted

Wave Loop 494 followed through on the W493 backend hardening by attempting the
first value-preservation theorem for the Icarus-lowerable scalar subset. The
goals were:

1. Define a denotational semantics for the simplified t27 AST over concrete
   bit-vectors.
2. Define a matching semantics for the shallow Verilog AST produced by the
   emitter model.
3. Prove that at least one representative witness computes the same value in
   t27 and in the emitted Verilog.
4. Keep the full repository gate green and produce three W495 cooperation
   variants.

---

## 2. What actually happened

### 2.1 Scalar semantics added

Created `proofs/lean4/Trinity/IcarusLowerable/Semantics.lean` with:

- `Value` (a `BitVec width` together with its width).
- `Valuation` as `String → Option Value`.
- `evalExpr / evalCall / evalFunction / evalStmts / evalTest` for the t27 AST.
- `evalVExpr / evalVStmt / evalVStmts / evalVModule` for the shallow Verilog AST.
- A shared `evalBinop` / `evalUnop` for numeric and boolean operators.
- Struct field access modeled as bit-vector slicing (`BitVec.extractLsb'`).
- Struct/array literals modeled as concatenation (`BitVec.append`).
- Function calls modeled by inlining the callee body.

The semantics is intentionally *combinational and finite*: it mirrors the
purely combinational subset that the current t27 → Verilog backend lowers.
Sequential constructs (alwaysComb, initial, forLoop) are kept as statement
lists for now.

### 2.2 Representative equivalence theorem

Added to `Soundness.lean`:

```lean
theorem scalar_struct_value_equiv :
  evalFunction scalarStructEnv scalarStructModule scalarStructMain []
    = evalVModule scalarStructEnv (emitModule scalarStructEnv scalarStructModule) := by
  native_decide
```

This proves that the scalar-struct-literal witness (`Pt { x: 1, y: 2 }`) is
packed into the same 16-bit value by the t27 evaluator and by the shallow
Verilog evaluator applied to the emitted module. It is the first
machine-checked value-preservation result for the Icarus-lowerable subset.

### 2.3 Scope limits

The generic theorem
`Module.isLowerable env m → evalModule env m = evalVModule env (emitModule env m)`
was not closed in this wave. The remaining work is:
- Model Verilog function bodies so that function-call equivalence can be proved
  (currently `evalVExpr` returns `none` for `.call`).
- Add `evalModule` for t27 that runs the whole module including test harness.
- Prove equivalence for the W493 witnesses that involve nested struct-return
  field access and struct-literal fields from identifiers.

These are explicitly queued as W495 follow-up.

---

## 3. Verification numbers

| Gate | Result |
|------|--------|
| `lake build Trinity.IcarusLowerable.*` | **green** |
| `./scripts/tri test --fast --icarus-lowerable` | **697 / 697 non-smoke PASS**, 0 seal mismatches, 0 Icarus disagreements |
| Icarus smoke | **176 / 177 PASS** (1 documented baseline failure: `w493_local_aos_element_field_not_lowerable.t27`) |
| Yosys smoke | **177 / 177 PASS** (0 baseline failures) |
| `cargo test -p t27c --bin t27c` | **1525 / 0 / 2** |
| `tri verify --lean-lowerable` | **green**, 253 specs in `Completeness.lean` |

No reseal was necessary because `bootstrap/src/compiler.rs` was not changed.

---

## 4. Files changed

- `proofs/lean4/Trinity/IcarusLowerable/Semantics.lean` — new scalar semantics for
  t27 and shallow Verilog.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean` — imports Semantics and
  adds the representative equivalence theorem.
- `proofs/lean4/Trinity.lean` — imports `Semantics` and `Soundness`.
- `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean` — regenerated (count
  unchanged at 253).
- `.claude/plans/wave-loop-494.md` — updated with completion markers.

---

## 5. Lessons for the next wave

1. **Bit-vector widths must be threaded explicitly.** Lean's `BitVec` family is
   indexed by width, so every helper (`extractLsb'`, `append`) carries a proof
   obligation that the slices line up. Using `extractLsb' start len` avoids the
   arithmetic normalization needed by `extractLsb`.
2. **Function calls are the next hard boundary.** The shallow Verilog AST stores
   `.call` nodes but not function bodies; to prove equivalence for
   struct-return-call witnesses we must either inline calls in the Verilog
   evaluator or store function definitions in `VModule`.
3. **Native_decide scales to concrete equivalence.** Once both sides are
   computable, checking equality on a concrete witness is automatic.

---

*φ² + φ⁻² = 3 | TRINITY*
