# Wave Loop 495 — Decomposed Plan

**Issue:** #1465  
**Branch:** `wave-loop-495`  
**Variant:** A — extend semantic equivalence to function calls and W493 witnesses  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Weak points identified

1. The shallow Verilog AST stores `.call` nodes but has no function definitions,
   so `evalVExpr` returns `none` for every function call. This blocks equivalence
   proofs for all W493 witnesses that rely on struct-return calls.
2. t27 `evalExpr` field access only recognizes constructor calls
   (`env.structForConstructor`). General function calls returning structs fall
   through to `none`.
3. `evalExpr` / `evalVExpr` array indexing hard-codes an 8-bit element width.
   It should derive the width from the array-typed base expression.
4. `evalVModule` flattens function bodies and returns `__return` from top-level
   items; there is no way to evaluate a specific Verilog `function` by name.
5. The generic equivalence theorem is not stated. We only have one witness
   theorem.
6. Module-level constants are not evaluated before a function runs in the
   model. The three scalar-struct witnesses work around this by inlining the
   constant value, but the AOS-element witness needs a module-level array
   literal.
7. Signed arithmetic, conditionals, and loops are outside the current
   combinational model; that boundary is acceptable for this wave but must be
   documented.

---

## Scientific background

- **Lutsig** (Andreas Lööw, CPP 2021). A verified Verilog-to-netlist compiler in
  HOL4 with a machine-checked semantic-equivalence theorem. Source and target
  both use a formal Verilog semantics; later work (FMCAD 2022) adds
  `always_comb` support. Relevant because it gives the canonical theorem shape
  we are aiming at: lowerability implies semantic preservation.
- **Vericert** (Yann Herklotz et al., OOPSLA 2021). A verified C-to-Verilog HLS
  compiler built on CompCert. It ports the Lööw–Myreen Verilog semantics to Coq,
  adapts it for BRAM/stack memories, and proves a backward-simulation theorem.
  Relevant because it shows how to integrate a bit-vector target semantics into
  a source-language simulation framework.
- **"The Essence of Verilog"** (Chen et al., OOPSLA 2023). A tested operational
  semantics (λV) for Verilog, validated against Icarus and Verilator. Relevant
  because it justifies treating our subset as finite and combinational, and
  because it documents real simulator ambiguities we must avoid.
- **Kami** (Choi et al., ICFP 2017). A Coq-embedded hardware DSL inspired by
  Bluespec; uses guarded atomic actions, trace inclusion, and extraction to
  Bluespec/Verilog. Relevant as a proof-assistant precedent for hardware DSL
  compilation with modular refinement.
- **Sparkle / Verilean** (github.com/Verilean/sparkle). A Lean 4 HDL compiler that
  generates SystemVerilog and verifies IP cores inside Lean. Relevant because it
  is the closest Lean-native analog to our goal.
- **CktFormalizer** (Xiong et al., arXiv 2026). LLM-generated hardware translated
  into a dependently typed Lean HDL, then to synthesizable Verilog with machine-
  checked equivalence. Relevant because it uses the same source-language proof
  assistant as the target model.
- **Melchert et al. (FMCAD 2025)**. SMT-based translation validation of a CGRA
  compiler across multiple IR stages including Verilog. Relevant because our
  per-witness `native_decide` proofs are a decidable, proof-assistant version of
  translation validation.

---

## Decomposition

### Phase 1 — AST / Emitter foundation (1–2 h)

- Add `VFunction` to `Verilog.lean`:
  ```lean
  structure VFunction where
    name : String
    params : List (String × Nat)
    retWidth : Nat
    body : List VStmt
  ```
- Extend `VModule` with `functions : List VFunction`.
- Update `Emitter.lean`:
  - `emitVFunction env fn` builds a `VFunction` from a t27 `Function`.
  - `emitModule env m` keeps module-level items for globals, emits functions as
    `VFunction` definitions, and still flattens tests/benches into items (they
    behave like `initial` blocks in the model).

### Phase 2 — Semantics extension (2–3 h)

- Add a type-inference helper for the lowerable subset:
  - `typeOfExpr env m val e : Option Ty` for identifiers, calls, field access,
    index, struct/array literals.
- Generalize `evalExpr .fieldAccess`:
  - If base is a call, use the callee's return type (function or constructor)
    to find the struct name.
  - If base is an identifier, look up its type from a new `Valuation` that
    carries both value and type, or from local declarations in the function.
- Generalize `evalExpr .index` and `evalVExpr .index` to derive `elemW` from
  the base expression's type.
- Add `evalVFunction` and update `evalVExpr .call` to inline a `VFunction`
  body.
- Change `evalVModule` to take a function name, evaluate module items first,
  then run the named `VFunction`.

### Phase 3 — Witness models and theorems (2–3 h)

- In `Lemmas.lean`, add four W495 witness environments and modules:
  - `w493NestedIdentifierEnv` / `Module` — `make_inner(5)` parameter used as a
    struct-literal field.
  - `w493LocalScalarEnv` / `Module` — local `inner` variable used as a field.
  - `w493ModuleScalarEnv` / `Module` — module-level `INNER_CONST` used as a
    field.
  - `w493ModuleAosEnv` / `Module` — literal-index element of a module-level
    array-of-struct constant used as a field.
- In `Soundness.lean`, prove `Module.isLowerable` for each (or reuse existing
  predicate) and prove value-preservation theorems with `native_decide`.

### Phase 4 — Generic theorem statement (1 h)

- State the top-level theorem:
  ```lean
  theorem module_value_equiv (env : Env) (m : Module)
    (h : Module.isLowerable env m)
    (mainFn : Function)
    (hm : m.findFunction "main" = some mainFn) :
    evalFunction env m mainFn [] =
    evalVModule env (emitModule env m) "main" := by
    sorry
  ```
- Prove the scalar base case (the existing W494 witness) and document that the
  function-call extension is covered by the witness set.

### Phase 5 — Validation (1–2 h)

- `lake build Trinity.IcarusLowerable.*`
- `cargo test -p t27c --bin t27c`
- `./scripts/tri test --fast`
- If the `Completeness.lean` count shifts because the predicate changed, run the
  exporter/regenerate the corpus file.

### Phase 6 — Close-out and W496 variants (1 h)

- `docs/reports/WAVE_LOOP_495_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W496_2026-07-13.md`
- Update `docs/NOW.md`, `.trinity/experience.md`, persistent memory.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| Changing `VModule` breaks existing soundness/semantics proofs | Update all call sites in `Emitter.lean`, `Semantics.lean`, `Soundness.lean` together; keep `Module.isLowerable` unchanged |
| `evalExpr` type inference becomes partial | Keep it a `partial def`; concrete witness terms are closed enough for `native_decide` |
| `lake build` times out on new theorems | Witness modules are tiny; use `native_decide` and avoid manual proof automation |
| `tri test --fast` fails due to unrelated change | No compiler changes planned; gate should stay green |
| Generic theorem cannot be fully proved | State it with `sorry` and prove the witness set; do not let perfect block good |

---

*φ² + φ⁻² = 3 | TRINITY*
