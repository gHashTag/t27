# Wave Loop 535 Plan — Align the Lean 4 lowerability predicate with the Rust structural classifier

**Issue:** #1506  
**Branch:** `wave-loop-535`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points audited

Wave Loop 534 made the Icarus lowerability boundary structural in Rust, but the
formal predicate in `Trinity.IcarusLowerable.Predicate.lean` still accepts two
families of constructs that the Rust classifier rejects:

1. **Unbounded `while (true)` loops.**  The Rust classifier rejects constant-true
   loop conditions because they cannot be lowered to a fuel-bounded Icarus
   simulation model.  Lean's `Stmt.isLowerableFuel` accepts any lowerable
   expression as a `whileLoop` condition, including `.boolLit true`.
2. **Non-lowerable struct fields.**  The Rust classifier rejects a scalar struct
   declaration if any field is `f32`, `string`, enum, or a nested non-lowerable
   struct.  Lean's `Ty.isLowerable` returns `true` for every `.struct _`
   unconditionally and only checks leaf lowerability when the field is accessed
   through a constructor-return call.

These gaps mean the Lean model is not yet a sound cross-check of the Rust gate.
If a future change accidentally broadens the Rust subset, the Lean proofs may
still pass even though the operational classifier changed.

A second weak point is the absence of machine-checkable negative theorems for
the six W534 adversarial witnesses.  The Rust integration test rejects them, but
the formal side only has positive witnesses.

---

## 2. Scientific literature surveyed

- **Vericert** (OOPSLA 2021) — a verified C-to-Verilog HLS compiler.  Its
  block-list / unsupported-construct predicate is the closest analog: a
  source-level check that guarantees the backend can emit synthesizable
  hardware.  The lesson is that the lowerability predicate must be defined on
  the *source* AST, not on generated code.
- **CompCert** (Leroy, JAR 2009) — the classic verified compiler uses a
  `transf_program` pass with an `OK` monad; constructs outside the supported
  subset are rejected early.  This matches the t27 philosophy of failing fast
  with a clear diagnostic.
- **"The Essence of Verilog"** (OOPSLA 2023) — a modern formal semantics of
  synthesizable Verilog.  It distinguishes *bounded* vs. *unbounded*
  procedural loops and shows that unbounded loops break standard simulation
  semantics unless guarded by an explicit fuel/timeout mechanism.
- **Kami** (POPL 2017) — a hardware description language embedded in Coq.  Its
  `WfMod` predicate checks structural well-formedness (no combinational loops,
  finite state) before allowing code generation.  This is the formal-model
  counterpart to t27's `Module.isLowerable`.

Key insight: every verified hardware pipeline needs an explicit, checkable
source predicate that is *tighter* than the backend's accidental acceptance
criteria.  W535 closes that tightening step.

---

## 3. Decomposed plan

### 3.1 Tighten `Predicate.lean`

- Add a structural check that `whileLoop` conditions are not constant `true`
  literals.
- Change `Ty.isLowerable` for `.struct name` to look up the field list in the
  environment and require every field to be lowerable (recursively, with fuel
  to keep the definition transparent).
- Keep positive witnesses green; any existing `Module.isLowerable` theorem that
  now fails must be fixed by correcting the environment, not by weakening the
  predicate.

### 3.2 Add negative theorems in `Lemmas.lean`

For each W534 negative witness, define a simplified AST environment + module
and state `¬ Module.isLowerable env m`:

| Witness | Formalization |
|---|---|
| `w534_negative_cast_to_string.t27` | `Expr.cast` to `.string` inside a function return. |
| `w534_negative_f32_field.t27` | Struct with an `.f32` field. |
| `w534_negative_host_only_helper.t27` | Call to a host-only function. |
| `w534_negative_nonlowerable_struct_assign.t27` | Variable declaration of a non-lowerable struct type. |
| `w534_negative_unbounded_while.t27` | `whileLoop` with `.boolLit true` condition. |
| `w534_negative_unresolved_import.t27` | Call to an undefined / imported function. |

All theorems discharged with `native_decide`.

### 3.3 Corpus positive witness with bounded `while`

- Create `specs/igla/w535_bounded_while_module.t27`: a module with a bounded
  `while (i < N)` loop and a value-preservation test that the Rust gate accepts.
- Add a simplified env/module pair and a `Module.isLowerable` theorem in
  `Completeness.lean` so the tightened predicate still accepts the lowerable
  subset.

### 3.4 Validation gates

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-simulate --icarus-lowerable --fast`
- `lake build Trinity.IcarusLowerable.Soundness`

---

## 4. Cooperation variants for Wave Loop 536

### Variant A (recommended): Equivalence-proof automation for the lowerability predicate

Add a Rust-to-Lean AST exporter so that the structural classifier can be run on
the exact simplified AST used by `Predicate.lean`.  Build a single end-to-end
test that asserts `Rust structural verdict = Lean Module.isLowerable verdict`
for every scratch witness and every corpus spec in `Completeness.lean`.  This
removes the last manual alignment step and makes divergence impossible to hide.

### Variant B: Cocotb reference-model cosimulation

Layer a Python cocotb model on top of the Icarus gate.  For each lowerable
witness, run the generated Verilog and an independent t27 interpreter in lock
step and compare outputs.  Catches value-level semantic drift that structural
lowerability alone cannot detect.

### Variant C: Sequential soundness for module-level initialization

Extend `Soundness.lean` with a source semantics for module-level `const`/`var`
procedural initialization and prove `module_value_equiv` for whole-struct
assignment at module scope.  Requires a non-scratch corpus witness in
`specs/igla/`.

---

*φ² + φ⁻² = 3 | TRINITY*
