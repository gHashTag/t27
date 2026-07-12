# Wave Loop 505 — Decomposed Plan

**Issue:** #1474 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-505`  
**Variant:** A — adversarial sequential witnesses  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Harden the sequential `if` / `for` boundary added by W504 with at least four
adversarial scratch witnesses, proving lowerability and value preservation for
each and keeping the Icarus smoke gate at zero documented baseline failures.

---

## Phases

### 1. OBSERVE — understand the W504 boundary
- Read `docs/reports/WAVE_LOOP_504_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W505_2026-07-07.md`.
- Replay the W504 proof: `Predicate.lean` sequential definitions,
  `Equivalence.lean` `P_forLoop`, `Soundness.lean` `w504_for_sum_value_equiv`.
- Confirm current baselines: `./scripts/tri test` and
  `./scripts/tri verify --lean-lowerable`.

### 2. SPEC — design adversarial witnesses
- `w505_nested_if.t27` — nested conditionals with different return arms.
- `w505_if_in_for.t27` — conditional update inside a bounded loop.
- `w505_for_var_range.t27` — loop bound is a function parameter.
- `w505_for_return.t27` — return value computed by a loop.
- Each spec must contain `test`/`invariant`/`bench` (L4 TESTABILITY).

### 3. TDD — write tests before classifier/emitter changes
- Add `test` blocks that exercise both the t27 interpreter and the generated
  Verilog host result.
- Run `./scripts/tri test` and record which witnesses pass smoke and which are
  classified lowerable.

### 4. CODE/IMPL — align classifier and emitter
- If a witness passes Icarus smoke but `Module.isLowerable` rejects it, extend
  `Predicate.lean` (e.g. sequential arms in nested `if`).
- If a witness is classified lowerable but smoke fails, fix `Emitter.lean` or
  the shallow semantics.
- Keep all changes inside the existing sequential predicate; do not broaden to
  `while` / `switch` (reserved for Variants B/C).

### 5. GEN — generate and inspect Verilog
- Run `tri gen` for each witness; verify the emitted Verilog contains packed
  vector-friendly `if` / `for` patterns.

### 6. SEAL — save deterministic hashes
- Run `t27c seal <spec> --save` for each new spec.

### 7. VERIFY — prove value preservation
- Add environments/modules in `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`.
- Add `native_decide` lowerability theorems in `Soundness.lean`.
- For at least one witness, apply the generic sequential theorem
  `module_value_equiv_proved_sequential` directly.
- Run `lake build Trinity.IcarusLowerable.Soundness`.

### 8. LAND — commit and hand off
- Commit to `wave-loop-505` with `Closes #1474`.
- Update `.trinity/current-issue.md`, `docs/NOW.md`,
  `.trinity/current_task/.commit_count`, and `session_log.jsonl`.
- Create `wave-loop-506` branch.

### 9. LEARN — capture experience
- Save new patterns (sequential predicate maintenance, nested control-flow
  witnesses, classifier/emitter disagreement triage) to `.trinity/experience.md`
  and persistent memory.

---

## Acceptance criteria

- At least four new adversarial sequential witnesses pass both the classifier
  and Icarus smoke.
- `./scripts/tri verify --lean-lowerable` reports zero disagreements.
- `./scripts/tri test`: non-smoke, yosys, Icarus, seal, and FPGA gates all clean.
- `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

*φ² + φ⁻² = 3 | TRINITY*
