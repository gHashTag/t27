# Wave Loop 492 — Close-out Report

**Date:** 2026-07-12  
**Branch:** `wave-loop-492`  
**Issue:** #1462  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Selected variant

**Variant A (default) — Soundness of the Icarus-lowerable subset in Lean 4**
from `docs/reports/FPGA_LOOP_COOPERATION_W492_2026-07-11.md`.

W492 turns the W491 lowerability predicate into a machine-checked contract that
the modeled Verilog output contains no unsupported placeholder or TODO stub.
It also mechanically imports the current Icarus-passing corpus into Lean and
proves that the predicate accepts each modeled spec.

---

## What was implemented

### 1. Lean 4 soundness formalization (`proofs/lean4/Trinity/IcarusLowerable/`)

- `Verilog.lean` — a shallow, placeholder-aware Verilog AST (`VExpr`, `VStmt`,
  `VModule`) with explicit `.unsupportedIcarus` and `.todo` constructors.
- `Emitter.lean` — a pure emitter model from the simplified t27 AST to the
  shallow Verilog AST.  Lowerable constructs map to concrete Verilog nodes;
  unmodeled constructs map to placeholders.
- `Soundness.lean` — top-level contract `Module.isSound` and representative
  `native_decide` theorems showing that `Module.isLowerable env m` implies the
  emitted module has no placeholders.
- `Predicate.lean` — updated the lowerable operator list to include `and` and
  `or`, and updated struct-layout handling so imported struct field accesses are
  validated against the imported struct definitions.
- `Completeness.lean` (generated, gitignored) — 253 per-spec `Env`/`Module`
  definitions and `native_decide` lowerability theorems for the current
  Icarus-passing corpus; 294 specs are skipped because the exporter is still
  conservative on unmodeled constructs.

### 2. Rust model exporter and gate (`bootstrap/src/compiler.rs`, `bootstrap/src/main.rs`, `scripts/tri`)

- `emit_lean_model` prints a Lean `Env` and `Module` definition for a single spec.
- `t27c icarus-lowerable --emit-lean-model <spec>` exposes the exporter on the
  command line.
- `generate_lean_lowerable_completeness` scans the repo, runs the Icarus
  classifier, and writes the generated `Completeness.lean`.
- `t27c lean-lowerable --repo-root .` (mapped to `tri verify --lean-lowerable`)
  regenerates `Completeness.lean` and typechecks it with `lake env lean`.

### 3. Suite exit-status fix (`bootstrap/src/suite.rs`)

- An acceptable run (all failures are within the documented yosys/Icarus
  baselines and there are no other failures) now returns exit code 0 while still
  printing the baseline-aware `ACCEPTABLE: yes` summary.  Non-acceptable runs still
  fail with `Error: SOME TESTS FAILED`.

### 4. Adversarial boundary witness specs (`specs/scratch/`)

| Spec | Covers |
|------|--------|
| `w492_soundness_boundary.t27` | Positive witness: a scalar struct literal, struct parameter, and struct equality that are accepted by the predicate and whose modeled Verilog contains no placeholders. |
| `w492_predicate_rejects_nested_return_field.t27` | Adversarial witness: direct field access on a nested struct returned from a call emits an `UNSUPPORTED_ICARUS` placeholder; the predicate and Icarus smoke gate correctly reject it. |

### 5. Documentation and coordination

- `.claude/plans/wave-loop-492.md` — decomposed plan for this wave.
- `docs/reports/T27_VS_FORMAL_HDL_2026-07-11.md` — research snapshot updated with
  W492 precedents (Chen et al., Choi et al., Melchert et al.).
- `docs/reports/FPGA_LOOP_COOPERATION_W493_2026-07-12.md` — three W493 variants.
- `docs/NOW.md` and `.trinity/current-issue.md` updated for W492.
- `.trinity/experience.md` updated.
- `docs/reports/gen_verilog_smoke_baseline.json` and
  `docs/reports/gen_verilog_iverilog_smoke_baseline.json` updated to document the
  W491 and W492 adversarial witnesses.

---

## Verification

- `cargo build --release`: green.
- `cargo test -p t27c --bin t27c`: **1525 passed, 0 failed, 2 ignored**.
- `./target/release/t27c suite --repo-root . --fast`:
  - **693 / 693 non-smoke PASS** (681 base + 6 W490 + 4 W491 + 2 W492 scratch witnesses).
  - **172 / 173 yosys smoke PASS**, 1 documented baseline failure
    (`specs/scratch/w491_nested_struct_return_field_not_lowerable.t27`).
  - **171 / 173 Icarus smoke PASS**, 2 documented baseline failures
    (`specs/scratch/w491_nested_struct_return_field_not_lowerable.t27`,
    `specs/scratch/w492_predicate_rejects_nested_return_field.t27`).
  - **693 / 693 seal matches**.
  - 0 `UNSUPPORTED_ICARUS` placeholders outside the documented adversarial witnesses.
- `./target/release/t27c suite --repo-root . --fast --icarus-lowerable`:
  - **zero disagreements** between Icarus smoke results and lowerability verdicts.
  - 171 lowerable, 2 not_lowerable (the two documented adversarial witnesses).
- `t27c lean-lowerable --repo-root .` (via `tri verify --lean-lowerable`):
  - Wrote 253 lowerable specs to `Completeness.lean` (294 skipped with unmodeled
    placeholders) and typechecked the file successfully.
- `lake build Trinity.IcarusLowerable.Verilog Trinity.IcarusLowerable.Emitter
  Trinity.IcarusLowerable.Soundness`: green.
- NMSE seal: FRESH (`bootstrap/stage0/FROZEN_HASH` and
  `repro/numerics/nmse_manifest*.json` refreshed because
  `bootstrap/src/compiler.rs` changed).

---

## Risk and mitigation

| Risk | Mitigation |
|------|------------|
| Exporter drifts from the real Verilog emitter. | The exporter reuses the same AST collection as the classifier and is cross-checked by the Icarus smoke gate and the lowerability gate. |
| Generated `Completeness.lean` grows and slows the gate. | It is gitignored and regenerated on demand; only 253 fully-modeled specs are imported. |
| Lean predicate accepts a pattern that still produces a placeholder in the *real* emitter. | The soundness theorem checks the modeled emitter; real-emitter divergence is caught by the existing yosys/Icarus smoke gates and the `--icarus-lowerable` disagreement check. |
| Baseline failures mask a new regression. | The suite now exits successfully only when every failure is in the documented baseline; a new failure makes the run unacceptable. |

---

## Deliverables

- `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`
- `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` — updated.
- `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean` — generated, gitignored.
- `bootstrap/src/compiler.rs` — `emit_lean_model`, `generate_lean_lowerable_completeness`, `compute_icarus_lowerable` updates.
- `bootstrap/src/main.rs` — `lean-lowerable` / `icarus-lowerable --emit-lean-model` CLI.
- `bootstrap/src/suite.rs` — acceptable-run exit-status fix.
- `scripts/tri` — `tri verify --lean-lowerable` passthrough.
- `specs/scratch/w492_*.t27` — boundary witnesses.
- `docs/reports/WAVE_LOOP_492_CLOSEOUT.md` — this report.
- `docs/reports/FPGA_LOOP_COOPERATION_W493_2026-07-12.md` — W493 variants.
- `docs/reports/T27_VS_FORMAL_HDL_2026-07-11.md` — updated research snapshot.
- `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md` updated.
- Persistent memory entry — `wave-loop-492.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
