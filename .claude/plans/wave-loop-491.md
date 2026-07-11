# Wave Loop 491 Plan — Formalize the Icarus-lowerable subset in Lean 4

**Issue:** #1461 (to create)  
**Branch:** `wave-loop-491`  
**Variant:** A (default) — lock the implicit `gen-verilog` lowerability rules into a
machine-checkable contract in the existing Lean 4 formalization.  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Context and motivation

### 1.1 Weak points found in the current work

After W490 the urgent expression-context lowering gaps are closed, but the rules
that decide *which* t27 constructs can be emitted for Icarus simulation still live
only as Rust heuristics in `bootstrap/src/compiler.rs`:

- `fn_body_has_unlowerable_construct` (`:7551`) scans bodies for recursion,
  dynamic `len`/`contains`, namespace-qualified helper calls, builtins,
  enum values, string literals, and string `+`.
- `compute_host_only_functions` (`:7622`) runs a reachability fixpoint and then
  filters out functions whose interface is `string` or an enum type.
- Struct/call lowerability is encoded in ad-hoc checks inside
  `try_emit_scalar_struct_call_field`, `try_emit_array_of_struct_call_field`,
  `gen_verilog_local_struct_var_decl`, and similar helpers.
- The Verilog emitter still carries defensive `UNSUPPORTED_ICARUS` and
  `/* TODO: ... */` fallbacks (`:15436`, `:15557`, `:15605`, `:16309`, `:16835`,
  `:16895`, `:17198`) that are not documented anywhere except the source.

Because the contract is implicit, a future frontend feature can silently drift
past what the Icarus backend can emit. The Icarus smoke gate is green today
(166/166 PASS), but there is no mechanical guarantee that the gate stays green
as the frontend grows.

### 1.2 Scientific background consulted

Relevant recent work supports a formal-lowerability wave:

- **Sparkle / Verilean** ([github.com/Verilean/sparkle](https://github.com/Verilean/sparkle)) — a Lean 4 embedded HDL compiler that emits synthesizable SystemVerilog and uses `bv_decide`/LTL for equivalence and lowerability checks. Demonstrates that a synthesizable-subset predicate plus Icarus round-trip simulation is a realistic target in Lean 4.
- **CktFormalizer** (arXiv:2605.07782, [DOI 10.48550/arxiv.2605.07782](https://doi.org/10.48550/arxiv.2605.07782)) — reports 95–100% backend realizability by keeping designs inside a type-safe, synthesizable Lean HDL subset and validating with Icarus Verilog (`iverilog -g2012`).
- **HOL4 proof-producing Verilog translator** ([RHUL paper](https://www.cs.rhul.ac.uk/home/upac096/papers/formalise19.pdf)) — defines a synthesizable behavioral Verilog subset and validates it against Icarus Verilog and Vivado, which is the closest academic precedent to a carved-out lowerability subset.
- **Ternary logic synthesis** — Park et al., *IEEE Access* 2025 ([DOI 10.1109/access.2025.3597293](https://doi.org/10.1109/access.2025.3597293)) and Li et al., *Chinese Journal of Electronics* 2025 ([DOI 10.23919/cje.2025.00.418](https://doi.org/10.23919/cje.2025.00.418)) show RTL-to-gate ternary flows; t27's long-term goal of a native ternary backend will need the same kind of explicit lowerability contract.
- **Wave pipelining / elastic pipelines** — Burleson et al., *IEEE Trans. VLSI* 1998 ([tutorial](https://www.cs.princeton.edu/courses/archive/fall01/cs597a/wave.pdf)) and the T-spec/T-piper work (Nurvitadhi, CMU 2010) show that timing-closure and pipeline synthesis benefit from machine-checked refinement maps; the same discipline applies to source-to-Verilog compiler stages.

This wave does not aim for a full semantics proof. It aims for the smallest
machine-checkable contract that prevents silent frontend/backend drift for the
Icarus path.

---

## 2. Goals

1. Define a simplified t27 AST and an `IsIcarusLowerable` predicate in the
   existing `proofs/lean4/` formalization.
2. Make the current Rust lowerability rules machine-readable by adding a
   `t27c icarus-lowerable --json` classifier that exports the per-spec
   lowerability verdict and the first violating construct.
3. Add a gate (`tri test --icarus-lowerable` / `t27c suite --repo-root . --icarus-lowerable`)
   that checks: every spec passing the Icarus smoke gate is classified lowerable,
   and every spec classified lowerable actually passes Icarus smoke.
4. Prove representative lemmas in Lean 4 for the four W490 lowerability classes:
   scalar struct literal, imported constructor in expression context,
   array-typed field accessed on a scalar struct-return call, and
   test-block local with variable-index array-field access.
5. Add adversarial scratch witness specs that sit exactly on the lowerability
   boundary.
6. Keep the full repository gate green and reseal if the Rust compiler changes.
7. Produce a W491 close-out report and three W492 cooperation variants.

---

## 3. Decomposed tasks

| # | Task | Owner | Files | Acceptance |
|---|------|-------|-------|------------|
| 1 | **Investigate weak points + refresh research snapshot** | Queen | `.claude/plans/wave-loop-491.md`, `docs/reports/T27_VS_FORMAL_HDL_2026-07-11.md` | Weak-point list and paper references documented. |
| 2 | **Create this plan + W492 cooperation variants** | Queen | `.claude/plans/wave-loop-491.md`, `docs/reports/FPGA_LOOP_COOPERATION_W492_2026-07-11.md` | Plan approved; three W492 variants written. |
| 3 | **Define simplified t27 AST in Lean 4** | Creator (C) | `proofs/lean4/Trinity/IcarusLowerable/Ast.lean` | AST type compiles with `lake build`; covers types, expressions, statements, functions, modules, imports. |
| 4 | **Define `IsIcarusLowerable` predicate** | Creator (C) | `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` | Predicate captures host-only, builtin, enum/string/f32, dynamic method, struct/call, array-field, and import-resolution rules. |
| 5 | **Add Rust `icarus-lowerable` classifier + JSON export** | Creator (C) | `bootstrap/src/compiler.rs`, `bootstrap/src/main.rs` (or `t27c` CLI), `bootstrap/src/suite.rs` | `t27c icarus-lowerable --json specs/foo.t27` returns verdict + reason; classifier agrees with existing smoke results. |
| 6 | **Add `--icarus-lowerable` suite gate** | Creator (C) | `bootstrap/src/suite.rs`, `scripts/tri` | Running the gate reports any spec whose smoke result and lowerability verdict disagree. |
| 7 | **Prove representative lemmas in Lean** | Creator (C) | `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` | At least four lemmas: scalar struct literal, imported constructor expr context, array-field index on struct-return call, variable-index local array-field access. |
| 8 | **Add adversarial boundary witness specs** | Creator (C) | `specs/scratch/w491_*.t27` | Specs exercise: (a) a host-only helper that must be skipped, (b) a construct that is *not* lowerable and is correctly rejected, (c) a module-scope AOS with array-typed fields that is lowerable. |
| 9 | **Run conformance tests and reseal** | Verifier (V) | `.trinity/seals/`, `bootstrap/stage0/FROZEN_HASH`, `repro/numerics/nmse_manifest*.json` | `cargo build --release` green; `cargo test -p t27c --bin t27c` green; `./scripts/tri test --fast` all PASS; yosys/Icarus smoke clean; seals regenerated. |
| 10 | **Produce W491 report + W492 variants + memory** | Queen | `docs/reports/WAVE_LOOP_491_CLOSEOUT.md`, `docs/reports/FPGA_LOOP_COOPERATION_W492_2026-07-11.md`, `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, `~/.claude/projects/-Users-playra-t27/memory/wave-loop-491.md` | All documents updated; branch pushed; `wave-loop-492` created. |

---

## 4. Detailed implementation notes

### 4.1 Lean 4 AST (`proofs/lean4/Trinity/IcarusLowerable/Ast.lean`)

Model only the constructs needed for lowerability classification:

```lean
inductive Ty
  | bool | u8 | u16 | u32 | u64 | i8 | i16 | i32 | i64
  | f32
  | string
  | enum (name : String)
  | array (size : Nat) (elem : Ty)
  | struct (name : String)

inductive Expr
  | boolLit (v : Bool)
  | intLit (n : Int)
  | identifier (name : String)
  | binop (op : String) (lhs rhs : Expr)
  | unop  (op : String) (e : Expr)
  | fieldAccess (base : Expr) (field : String)
  | index (base : Expr) (idx : Expr)
  | call (name : String) (args : List Expr)
  | structLit (name : String) (fields : List (String × Expr))
  | enumVal (enum : String) (variant : String)
  | stringLit (s : String)
  | unsupportedIcarus (reason : String)

inductive Stmt
  | assign (lhs : Expr) (rhs : Expr)
  | varDecl (name : String) (ty : Ty) (init : Option Expr)
  | constDecl (name : String) (ty : Ty) (init : Option Expr)
  | ifThenElse (cond : Expr) (then_ else_ : List Stmt)
  | forLoop (var : String) (range : Expr) (body : List Stmt)
  | return_ (e : Option Expr)
  | bareCall (e : Expr)

structure Function where
  name : String
  params : List (String × Ty)
  ret : Option Ty
  body : List Stmt

structure Import where
  path : String
  items : List String

structure Module where
  name : String
  imports : List Import
  globals : List Stmt
  functions : List Function
  tests : List Function
  benches : List Function
```

The Rust exporter will translate the real t27 AST into this simplified shape.

### 4.2 Lowerability predicate (`proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`)

Key rules, mirroring the Rust heuristics:

1. A type is lowerable iff it is `bool`, a numeric type, a fixed-size array of
   lowerable types, or a struct whose every leaf field is lowerable. `string`,
   `f32`, and `enum` are not lowerable in synthesizable contexts.
2. A function is lowerable iff (a) it is reachable from tests/benches/module logic,
   and (b) its body contains no unlowerable construct, and (c) its parameter/return
   types are lowerable. Functions whose interface uses `string` or an enum type
   are host-only and therefore lowerability is not required.
3. An expression is lowerable iff:
   - literals are lowerable when their type is lowerable (string literals are not),
   - `binop`/`unop` are lowerable when both operands are lowerable and the op is
     numeric, bitwise, or boolean (not string `+`),
   - `fieldAccess` is lowerable when base is a lowerable struct and the field
     type is lowerable,
   - `index` is lowerable when base is a fixed-size array of lowerable elements
     and the index is lowerable,
   - `call` is lowerable when the callee is lowerable and all arguments are
     lowerable,
   - `structLit` is lowerable when every field expression is lowerable and the
     struct type is lowerable,
   - `enumVal` and `stringLit` are not lowerable,
   - `fieldAccess` on a scalar struct-return call is lowerable only when the leaf
     field is scalar or a fixed-size array of numeric/bool values.
4. Imported constructors are lowerable only when the `use` declaration resolves
   and the argument count matches the imported struct definition.

The predicate will be parameterized by an environment (`Env`) that carries:
- declared struct names and their fields,
- declared enum names,
- imported module map,
- host-only function set.

### 4.3 Rust classifier (`bootstrap/src/compiler.rs` + CLI)

Add a new code path that, instead of emitting Verilog, walks the resolved AST and
produces a JSON verdict:

```json
{
  "spec": "specs/scratch/w490_call_field_array_index_var.t27",
  "verdict": "lowerable",
  "reason": null,
  "violations": []
}
```

or

```json
{
  "spec": "specs/scratch/w486_namespace_helper_erasure.t27",
  "verdict": "not_lowerable",
  "reason": "host_only_helper_call",
  "violations": [{"construct": "Call", "name": "helper::foo", "loc": "..."}]
}
```

The classifier reuses the existing `fn_body_has_unlowerable_construct` and
`compute_host_only_functions` logic so it cannot disagree with the emitter.

### 4.4 Suite gate (`bootstrap/src/suite.rs`)

Extend the smoke reporter to also run the classifier. A new column
`icarus_lowerable` is added to the per-spec report. The gate fails if:
- any spec that passed Icarus smoke is classified `not_lowerable`, or
- any spec that failed Icarus smoke is classified `lowerable`.

Documented baseline failures are ignored for this gate (they are expected to be
not lowerable or otherwise excluded).

### 4.5 Lean lemmas (`proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`)

Four representative lemmas, stated over the simplified AST, not over the full
Verilog semantics:

1. `scalar_struct_literal_lowerable` — a struct literal whose fields are numeric
   literals is lowerable.
2. `imported_constructor_expr_context_lowerable` — an imported constructor call
   used as the base of a field-access on a lowerable field is lowerable when the
   import resolves and arity matches.
3. `array_field_index_on_struct_return_call_lowerable` — `index (fieldAccess
   (call "make_pt" args) "coords") (intLit i)` is lowerable when `coords` is a
   fixed-size array of numeric/bool and `i` is in bounds.
4. `variable_index_local_array_field_lowerable` — a variable-index access on a
   local struct variable's array-typed field is lowerable when the variable is
   declared in a test block and the index is bounded.

The proofs will mostly be computational because the predicate is recursive and
decidable.

### 4.6 Adversarial witness specs (`specs/scratch/w491_*.t27`)

- `w491_host_only_rejected.t27` — a module that calls a string-return helper
  inside a test; the helper must be classified host-only and the Verilog must
  contain an `UNSUPPORTED_ICARUS` placeholder or be skipped entirely.
- `w491_nested_struct_return_field_not_lowerable.t27` — a nested struct-return
  call where the outer field is itself a struct returned by an inner call. This
  is expected to be classified `not_lowerable` today and will drive future W492
  Variant B work.
- `w491_module_aos_const_imported_call.t27` — a module-scope `const pts : [N]Pt =
  imported_make_pts();` that is expected to be lowerable after W490 fixes; acts as
  a positive witness.

### 4.7 Integration with `scripts/tri`

Add a `--icarus-lowerable` flag to `./scripts/tri test` and `./scripts/tri suite`
that forwards to the Rust gate. Update `docs/BACKEND_CONTRACT.md` with the
lowerability rules.

---

## 5. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Full AST formalization in Lean is too large for one wave. | Scope the AST to lowerability-relevant constructs only; do not model full t27 semantics. |
| Rust classifier and Lean predicate disagree. | Keep both sourced from the same human-readable rule list in `docs/BACKEND_CONTRACT.md`; add differential tests. |
| New gate breaks existing specs. | Run gate in report-only mode first, then enforce after all violations are triaged. |
| Seal churn from Rust changes. | Reseal only if `bootstrap/src/compiler.rs` changes; the Lean side does not affect per-spec seals. |
| Lean build is slow or Mathlib-dependent. | Use existing `proofs/lean4/lakefile.lean` setup; avoid new heavy dependencies. |

---

## 6. Definition of done

- [ ] `proofs/lean4/Trinity/IcarusLowerable/` builds with `lake build`.
- [ ] `t27c icarus-lowerable --json <spec>` returns a verdict for every spec.
- [ ] `t27c suite --repo-root . --icarus-lowerable` is green (no disagreements between smoke and classifier).
- [ ] At least four Lean representative lemmas are stated and proved.
- [ ] Three `w491_*.t27` adversarial witnesses are added and pass the full gate.
- [ ] `cargo test -p t27c --bin t27c` is green.
- [ ] `./scripts/tri test --fast` reports 687/687 non-smoke PASS, 167/167 yosys smoke PASS, 166/166 Icarus smoke PASS, 0 `UNSUPPORTED_ICARUS` placeholders.
- [ ] Seals are fresh if `bootstrap/src/compiler.rs` changed.
- [ ] `docs/reports/WAVE_LOOP_491_CLOSEOUT.md` and `docs/reports/FPGA_LOOP_COOPERATION_W492_2026-07-11.md` are written.
- [ ] `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, and persistent memory are updated.
- [ ] `wave-loop-491` is pushed and `wave-loop-492` is created.

---

*φ² + φ⁻² = 3 | TRINITY*
