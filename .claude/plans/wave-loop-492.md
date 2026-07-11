# Wave Loop 492 Plan — Soundness of the Icarus-lowerable subset in Lean 4

**Issue:** #1462 (to create)  
**Branch:** `wave-loop-492`  
**Variant:** A — extend the W491 formalization into a machine-checked soundness
claim for the Icarus-lowerable subset, plus a completeness import of the current
corpus.  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Context and motivation

### 1.1 Weak points found in the current work

Wave Loop 481 closed the last urgent functional holes in the `gen-verilog`
backend and W491 carved out an explicit Icarus-lowerability predicate in Lean 4.
The classifier now agrees with the Icarus smoke gate on every spec
(170/170 lowerable, 1 documented adversarial baseline failure, zero
disagreements).  However, two important guarantees are still implicit:

1. **Soundness.** The Lean predicate says *which* t27 constructs are lowerable,
   but it does not yet say *what* the backend emits for those constructs.  A
   future change could make the predicate accept a pattern that the Verilog
   emitter lowers with an `UNSUPPORTED_ICARUS` or `// TODO: implement` stub.
2. **Completeness over the corpus.** The classifier is tested against the smoke
   gate, but the corpus verdicts are not mechanically imported into the Lean
   proof.  A regression in the classifier could remove a lowerable spec from the
   predicate without breaking the Lean build.

The weak points are therefore:

- `bootstrap/src/compiler.rs:18482` — `IcarusLowerabilityResult` carries a
  verdict, but the connection to the emitted Verilog is only checked by the
  Icarus smoke pass, not by the formal proof.
- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` — the predicate is
  decidable but isolated from the actual output of the emitter.
- `docs/BACKEND_CONTRACT.md` — section 4.1 lists the lowerable subset, but the
  document is not backed by a machine-checked proof that the backend honors it.

### 1.2 Scientific background consulted

The W492 work is grounded in recent hardware-compiler verification literature:

- **Sparkle / Verilean** ([github.com/Verilean/sparkle](https://github.com/Verilean/sparkle))
  — a Lean 4 embedded HDL that emits synthesizable SystemVerilog and uses
  Icarus round-trip simulation as an oracle.  It demonstrates that a
  synthesizable-subset predicate plus a shallow emitter model is a realistic
  Lean target.
- **CktFormalizer** (arXiv:2605.07782,
  [DOI 10.48550/arxiv.2605.07782](https://doi.org/10.48550/arxiv.2605.07782))
  — uses a dependently typed Lean 4 HDL as a "correctness firewall" so that
  compiled designs are structurally free of defects that silently fail in
  backend flows.  The key takeaway for W492 is that the lowerability predicate
  should guarantee absence of placeholder/unsupported constructs in the emitted
  RTL.
- **HOL4 proof-producing Verilog translator** ([RHUL paper](https://www.cs.rhul.ac.uk/home/upac096/papers/formalise19.pdf))
  — defines a behavioral synthesizable Verilog subset and validates it against
  Icarus Verilog.  The closest academic precedent to a carved-out lowerability
  subset with an oracle simulator.
- **The Essence of Verilog** (Chen et al., OOPSLA 2023,
  [paper](https://cs.nju.edu.cn/yueli/papers/oopsla2023.pdf)) — a modern
  operational semantics for Verilog tested against Icarus.  W492 does not model
  the full semantics, but it adopts the same Icarus-as-oracle discipline.
- **Revamping Verilog Semantics for Foundational Verification** (Choi et al.,
  POPL 2025, [DOI 10.1145/3763084](https://doi.org/10.1145/3763084)) — a recent
  Coq/Rocq foundational semantics for the synthesizable subset.  Shows that
  small, well-scoped semantics are enough to lock in compiler contracts.
- **Automated Translation Validation of a Compiler for Statically Scheduled
  Accelerators** (Melchert et al., FMCAD 2025,
  [PDF](https://repositum.tuwien.at/bitstream/20.500.12708/219556/1/Melchert%20Jackson%20-%202025%20-%20Automated%20Translation%20Validation%20of%20a%20Compiler%20for...pdf))
  — SMT-based translation validation across every compiler stage down to Verilog
  RTL, using Yosys to lift RTL into an SMT transition system.  Relevant as a
  precedent for machine-checked backend validation, even though W492 uses a
  lightweight predicate-based approach rather than full equivalence checking.
- **FIRRTL ABI** ([abi.md](https://github.com/chipsalliance/firrtl-spec/blob/main/abi.md))
  — documents the contract between an intermediate language and its SystemVerilog
  backend.  t27 needs the same kind of explicit contract to Icarus.

This wave does **not** prove full semantic correctness of the t27 → Verilog
compiler.  It proves the smallest meaningful contract: *the Icarus-lowerability
predicate guarantees that the modeled Verilog output contains no unsupported
placeholder or TODO stub*.

---

## 2. Goals

1. Define a shallow, placeholder-aware Verilog AST in Lean 4.
2. Define a pure emitter model from the simplified t27 AST to the shallow
   Verilog AST.
3. Prove a soundness theorem: if a t27 module is accepted by the Icarus
   lowerability predicate, then the modeled Verilog contains no
   `UNSUPPORTED_ICARUS` or `// TODO` placeholders.
4. Mechanically import the lowerability verdicts of the current Icarus-passing
   corpus into Lean and prove that the predicate accepts them.
5. Add a `tri verify --lean-lowerable` gate (or equivalent) that runs the
   soundness proof and the completeness import, failing on any regression.
6. Add one adversarial scratch witness that exercises the soundness boundary
   (a construct accepted by a naïve predicate but rejected by the real emitter).
7. Keep the full repository gate green and reseal if the Rust compiler or the
   generated model changes.
8. Produce a W492 close-out report and three W493 cooperation variants.

---

## 3. Decomposed tasks

| # | Task | Owner | Files | Acceptance |
|---|------|-------|-------|------------|
| 1 | **Refresh weak-point + research snapshot** | Queen | `.claude/plans/wave-loop-492.md`, `docs/reports/T27_VS_FORMAL_HDL_2026-07-11.md` (update) | Weak points and paper references documented for W492. |
| 2 | **Create this plan + W493 cooperation variants** | Queen | `.claude/plans/wave-loop-492.md`, `docs/reports/FPGA_LOOP_COOPERATION_W493_2026-07-*.md` | Plan approved; three W493 variants written. |
| 3 | **Shallow Verilog AST in Lean 4** | Creator (C) | `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean` | AST compiles with `lake build`; includes placeholder constructors. |
| 4 | **Pure emitter model** | Creator (C) | `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean` | Every lowerable t27 construct maps to a concrete Verilog AST node; non-lowerable constructs map to placeholders. |
| 5 | **Soundness theorem** | Creator (C) | `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean` | `Module.isLowerable env m → emitModule env m = some v → ¬ hasPlaceholder v` is proved for the predicate. |
| 6 | **Rust model exporter** | Creator (C) | `bootstrap/src/compiler.rs`, `bootstrap/src/main.rs` | New `t27c icarus-lowerable --emit-lean-model <spec>` prints a Lean `Env` and `Module` definition for the spec. |
| 7 | **Completeness import generator** | Creator (C) | `bootstrap/src/compiler.rs` (or new `bootstrap/src/lean_lowerable.rs`) | Script/module that runs the exporter over all Icarus-passing specs and writes `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean`. |
| 8 | **`tri verify --lean-lowerable` gate** | Creator (C) | `bootstrap/src/main.rs`, `bootstrap/src/suite.rs`, `scripts/tri` | Command regenerates `Completeness.lean`, builds the Lean library, and reports any failure. |
| 9 | **Adversarial boundary witness** | Creator (C) | `specs/scratch/w492_*.t27` | One positive witness for the soundness theorem and one adversarial witness that the predicate rejects. |
| 10 | **Run conformance tests and reseal** | Verifier (V) | `.trinity/seals/`, `bootstrap/stage0/FROZEN_HASH`, `repro/numerics/nmse_manifest*.json` | `cargo build --release` green; `cargo test -p t27c --bin t27c` green; `./scripts/tri test --fast --icarus-lowerable` all PASS; `lake build Trinity.IcarusLowerable.*` green; seals regenerated. |
| 11 | **Produce W492 report + W493 variants + memory** | Queen | `docs/reports/WAVE_LOOP_492_CLOSEOUT.md`, `docs/reports/FPGA_LOOP_COOPERATION_W493_2026-07-*.md`, `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, memory | All documents updated; branch pushed; `wave-loop-493` created. |

---

## 4. Detailed implementation notes

### 4.1 Shallow Verilog AST (`proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`)

Model only the constructs that the current t27 → Icarus path emits, plus
explicit placeholders:

```lean
inductive VExpr
  | lit (width : Nat) (value : String)    -- decimal/bit vector literal
  | ident (name : String)                 -- wire/reg/parameter identifier
  | binop (op : String) (lhs rhs : VExpr)
  | unop (op : String) (e : VExpr)
  | index (base : VExpr) (idx : VExpr)
  | slice (base : VExpr) (hi lo : Nat)
  | concat (parts : List VExpr)
  | unsupported (reason : String)         -- UNSUPPORTED_ICARUS placeholder
  | todo (stub : String)                  -- // TODO: implement stub

inductive VStmt
  | assign (lhs : VExpr) (rhs : VExpr)
  | localparam (name : String) (width : Nat) (init : VExpr)
  | wire (name : String) (width : Nat)
  | reg (name : String) (width : Nat)
  | alwaysComb (body : List VStmt)
  | initial (body : List VStmt)
  | taskCall (name : String) (args : List VExpr)

structure VModule where
  name : String
  ports : List (String × Nat × String)  -- name, width, direction
  items : List VStmt
```

### 4.2 Emitter model (`proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`)

Define a function

```lean
def emitModule (env : Env) (m : Module) : Option VModule
```

that:

- succeeds only when every reachable function and global declaration is
  lowerable (this is already enforced by the predicate, so the emitter can be
  partial);
- translates lowerable types to bit widths using the existing struct-field
  registry;
- emits `unsupported` / `todo` nodes only when a construct is not in the
  lowerable subset.

The emitter does not need to produce bit-exact output; it only needs to preserve
enough structure to show that no placeholder is produced for lowerable input.

### 4.3 Soundness theorem (`proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`)

```lean
def VModule.hasPlaceholder (v : VModule) : Bool :=
  v.items.any VStmt.hasPlaceholder

theorem lowerable_implies_placeholder_free (env : Env) (m : Module)
    (h : Module.isLowerable env m)
    (he : emitModule env m = some v) :
    ¬ v.hasPlaceholder := by
  native_decide   -- or structural induction, depending on the final shape
```

The theorem is computational because the predicate, emitter, and placeholder
predicate are all recursive and decidable.  For the full corpus, the
`native_decide` path is expected to scale because each spec is small.

### 4.4 Rust model exporter

Add a new option to the existing `t27c icarus-lowerable` command:

```bash
t27c icarus-lowerable specs/foo.t27 --emit-lean-model
```

Output (Lean source):

```lean
def foo_env : Env := { structs := [...], constructors := [...], ... }
def foo_module : Module := { name := "foo", ... }
```

The exporter reuses the same AST collection and environment-building logic that
`compute_icarus_lowerable` already performs, but serializes the simplified
`Module` and `Env` into Lean syntax instead of producing a JSON verdict.

### 4.5 Completeness import generator

A Rust helper (either embedded in `compiler.rs` or a standalone module) that:

1. Collects all `.t27` specs under `specs/` and `specs/scratch/`.
2. Runs the Icarus smoke gate to determine which specs currently pass.
3. For each passing spec, runs the model exporter to obtain a Lean `Env` and
   `Module`.
4. Writes `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean` containing:
   - one `def` per spec for its `Env` and `Module`;
   - one theorem per spec: `Theorem <spec>_lowerable : Module.isLowerable <env> <module> := by native_decide`;
   - a top-level `theorem corpus_is_lowerable` that collects all per-spec
     theorems into a conjunction.

Because the generated file is large, it should be gitignored or regenerated on
demand.  The gate in step 8 regenerates it before building.

### 4.6 `tri verify --lean-lowerable` gate

Two possible shapes:

- **Separate command:** `t27c lean-lowerable --repo-root .` that regenerates
  `Completeness.lean`, runs `lake build Trinity.IcarusLowerable.Completeness`,
  and fails if any proof fails.
- **Suite flag:** extend `t27c suite --repo-root . --lean-lowerable` to run the
  regeneration + build as a new phase.

Use the separate command for W492 to avoid perturbing the already-green suite
flow.  `tri verify --lean-lowerable` maps to it via `scripts/tri` passthrough.

### 4.7 Adversarial witness specs

- `specs/scratch/w492_soundness_boundary.t27` — a spec that is accepted by the
  predicate and whose modeled Verilog has no placeholders, used as a positive
  regression test for the soundness theorem.
- `specs/scratch/w492_predicate_rejects_placeholder.t27` — a spec that a naïve
  predicate might accept but the real predicate rejects because the construct
  lowers to an `UNSUPPORTED_ICARUS` placeholder.  This drives the soundness
  boundary in the opposite direction.

### 4.8 Integration with existing gates

The new Lean modules must not break the existing `lake build Trinity` default
target.  Add the new imports to `Trinity.lean` only after the generated
`Completeness.lean` is present or guarded by conditional imports.

---

## 5. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Modeling the full Verilog emitter in Lean is too large. | Keep the Verilog AST shallow; emit only enough structure to detect placeholders. |
| Generated `Completeness.lean` is huge and slows `lake build`. | Generate per-spec theorems but let `native_decide` run in parallel; if necessary, gate runs only on `--lean-lowerable`, not on every `tri test`. |
| Rust model exporter drifts from the real emitter. | The exporter reuses the same AST collection as the classifier; differential tests compare exporter output against a few hand-written specs. |
| Lean build fails because generated file is missing. | Gate regenerates the file before `lake build`; CI calls the gate explicitly. |
| New gate breaks existing specs. | `--lean-lowerable` is optional in W492; make it mandatory only after it is green. |
| Seal churn. | Reseal only if `bootstrap/src/compiler.rs` or generated code changes. |

---

## 6. Definition of done

- [ ] `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`, `Emitter.lean`, and
      `Soundness.lean` build with `lake build`.
- [ ] `t27c icarus-lowerable --emit-lean-model <spec>` prints a valid Lean
      `Env` + `Module` for the spec.
- [ ] `Completeness.lean` is regenerated and proves lowerability for the
      current Icarus-passing corpus.
- [ ] `tri verify --lean-lowerable` (or equivalent) runs green.
- [ ] `./scripts/tri test --fast --icarus-lowerable` reports no disagreements.
- [ ] `cargo test -p t27c --bin t27c` is green.
- [ ] New scratch witness specs are added and pass the full gate.
- [ ] Seals are fresh if `bootstrap/src/compiler.rs` changed.
- [ ] `docs/reports/WAVE_LOOP_492_CLOSEOUT.md` and
      `docs/reports/FPGA_LOOP_COOPERATION_W493_2026-07-*.md` are written.
- [ ] `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, and
      persistent memory are updated.
- [ ] `wave-loop-492` is pushed and `wave-loop-493` is created.

---

*φ² + φ⁻² = 3 | TRINITY*
