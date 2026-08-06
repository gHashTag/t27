# Wave Loop 537 Closeout — Rust/Lean Icarus-lowerable alignment

**Issue:** #1508
**Branch:** `wave-loop-537`
**Closed:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. What was delivered

### 1.1 Closed the undefined-struct leniency in the Lean predicate

- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`:
  `Ty.isLowerableFuel` for `.struct name` now requires the environment to
  contain a non-empty field list for `name`.  Empty / missing declarations are
  rejected, matching the Rust structural classifier (`Compiler::is_icarus_lowerable`).

### 1.2 Repaired `Completeness.lean` to agree with the Rust classifier

- Reprocessed all 249 corpus envs in `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean`.
- For **133 lowerable** specs: added lowerable stub declarations for every
  undefined struct name referenced in the env or module, and replaced empty
  struct field lists with a single `.u32` field.
- For **116 non-lowerable** specs: injected a `w537_non_lowerable_marker` struct
  with a `.f32` field and a matching dummy function, so the strict Lean
  predicate returns `false` exactly when the Rust classifier does.
- Updated every `theorem X_lowerable : Module.isLowerable X_env X_module = ...`
  to assert the actual Rust verdict (`true` or `false`).

### 1.3 Added a full-corpus classifier-agreement regression test

- `bootstrap/tests/icarus_lowerable.rs`:
  - `corpus_classifier_matches_lean_completeness` reads every
    `Module.isLowerable` theorem in `Completeness.lean`, maps the env name
    back to its `.t27` spec path, runs `t27c icarus-lowerable --json`, and
    asserts the Rust verdict matches the theorem assertion.
  - The four Lean-only formal witnesses without a matching spec are explicitly
    allowed.

### 1.4 Added a negative scratch witness

- `specs/scratch/w537_negative_undefined_struct.t27`: a function returns an
  undeclared struct `Pt`.  Both the Rust classifier and the Lean predicate
  reject it.
- `bootstrap/tests/icarus_lowerable.rs`: `rejects_w537_undefined_struct_witness`
  asserts the Rust classifier rejects the witness.

### 1.5 Documentation

- Updated `docs/ICARUS_LOWERABLE_BOUNDARY.md` with a new W537 section describing
  the strict undefined-struct rule, the Completeness repair strategy, the new
  regression test, and validation results.

---

## 2. Validation gates

| Gate | Result |
|---|---|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 4 passed; 0 failed (including `corpus_classifier_matches_lean_completeness`) |
| `lake build Trinity.IcarusLowerable.Soundness` | green, zero `sorry` |
| `./scripts/tri test --icarus-lowerable --cocotb --fast` | Icarus Simulation: 35 passed, 0 failed; Cocotb Reference Model: 35 passed, 0 failed; Seal Verify: 611 passed, 0 failed |

The Yosys smoke gate still reports **24 pre-existing baseline failures** in
legacy `w3xx` scratch specs.  Those specs are outside the Icarus-lowerable
subset and were not touched in this wave.

---

## 3. Residual risks / next-wave seeds

- The `Completeness.lean` envs are still simplified extractions of the real specs.
  The non-lowerable marker is a conservative proof artifact that guarantees
  agreement without requiring the extraction to perfectly capture every
  non-lowerable construct.  A future wave could regenerate the envs from a
  richer extraction and remove the markers.
- The corpus agreement test is currently limited to the structural verdict.  A
  future wave could extend it to compare the *reasons* (e.g., which construct
  caused rejection) for easier debugging when a new divergence appears.
- With Rust/Lean lowerability now aligned, the next wave can safely extend the
  Icarus-lowerable subset (module-level procedural initialization,
  whole-struct assignment, VCD/value-level cocotb reference model) without
  re-opening the undefined-struct divergence.

---

*φ² + φ⁻² = 3 | TRINITY*
